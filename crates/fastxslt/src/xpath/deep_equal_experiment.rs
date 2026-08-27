use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeepEqualExpression {
    left: NodeSelection,
    right: NodeSelection,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeSelection {
    DescendantAttribute {
        element: String,
        position: usize,
        attribute: String,
    },
    DescendantComment {
        position: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeepEqualFailure {
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<DeepEqualExpression, DeepEqualFailure> {
    let arguments = expression
        .strip_prefix("deep-equal(")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(|value| value.split_once(','))
        .ok_or_else(|| unsupported(expression, location))?;
    Ok(DeepEqualExpression {
        left: parse_selection(arguments.0.trim(), location)?,
        right: parse_selection(arguments.1.trim(), location)?,
        location: location.clone(),
    })
}

fn parse_selection(
    expression: &str,
    location: &SourceLocation,
) -> Result<NodeSelection, DeepEqualFailure> {
    if let Some(position) = expression
        .strip_prefix("//comment()[")
        .and_then(|value| value.strip_suffix(']'))
        .and_then(parse_position)
    {
        return Ok(NodeSelection::DescendantComment { position });
    }
    let (element, attribute) = expression
        .strip_prefix("//")
        .and_then(|value| value.split_once("/@"))
        .ok_or_else(|| unsupported(expression, location))?;
    let (element, position) = parse_positioned_name(element)
        .filter(|(name, _)| is_ascii_ncname(name))
        .ok_or_else(|| unsupported(expression, location))?;
    if !is_ascii_ncname(attribute) {
        return Err(unsupported(expression, location));
    }
    Ok(NodeSelection::DescendantAttribute {
        element: element.to_owned(),
        position,
        attribute: attribute.to_owned(),
    })
}

fn parse_positioned_name(value: &str) -> Option<(&str, usize)> {
    let (name, position) = value.split_once('[')?;
    let position = position.strip_suffix(']').and_then(parse_position)?;
    Some((name, position))
}

fn parse_position(value: &str) -> Option<usize> {
    value.parse::<usize>().ok().filter(|position| *position > 0)
}

fn unsupported(expression: &str, location: &SourceLocation) -> DeepEqualFailure {
    DeepEqualFailure {
        detail: format!(
            "the private deep-equal slice requires positioned descendant attributes or comments: {expression}"
        ),
        location: location.clone(),
    }
}

pub(crate) fn evaluate(
    expression: &DeepEqualExpression,
    document: &Document,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let left = select_nodes(&expression.left, document, control)?;
    let right = select_nodes(&expression.right, document, control)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.into_iter().zip(right) {
        control.charge(WorkDomain::XPathOperation, 1)?;
        if !nodes_deep_equal(document, left, right) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn select_nodes(
    selection: &NodeSelection,
    document: &Document,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let mut selected = Vec::new();
    select_below(
        selection,
        document,
        document.document_node(),
        &mut selected,
        control,
    )?;
    Ok(selected)
}

fn select_below(
    selection: &NodeSelection,
    document: &Document,
    parent: NodeId,
    selected: &mut Vec<NodeId>,
    control: &mut InvocationControl,
) -> Result<(), ControlFailure> {
    let children = document.children(parent);
    let mut matching_children = Vec::new();
    for child in children.iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if matches_child(selection, document, child) {
            matching_children.push(child);
        }
    }
    let position = match selection {
        NodeSelection::DescendantAttribute { position, .. }
        | NodeSelection::DescendantComment { position } => *position,
    };
    if let Some(node) = matching_children.get(position - 1).copied() {
        match selection {
            NodeSelection::DescendantAttribute { attribute, .. } => {
                for candidate in document.attributes(node).iter().copied() {
                    control.charge(WorkDomain::XPathNodeVisit, 1)?;
                    if document
                        .name(candidate)
                        .is_some_and(|name| name.namespace.is_none() && name.local == *attribute)
                    {
                        selected.push(candidate);
                    }
                }
            }
            NodeSelection::DescendantComment { .. } => selected.push(node),
        }
    }
    for child in children.iter().copied() {
        if document.kind(child) == NodeKind::Element {
            select_below(selection, document, child, selected, control)?;
        }
    }
    Ok(())
}

fn matches_child(selection: &NodeSelection, document: &Document, node: NodeId) -> bool {
    match selection {
        NodeSelection::DescendantAttribute { element, .. } => {
            document.kind(node) == NodeKind::Element
                && document
                    .name(node)
                    .is_some_and(|name| name.namespace.is_none() && name.local == *element)
        }
        NodeSelection::DescendantComment { .. } => document.kind(node) == NodeKind::Comment,
    }
}

fn nodes_deep_equal(document: &Document, left: NodeId, right: NodeId) -> bool {
    document.kind(left) == document.kind(right)
        && document.name(left) == document.name(right)
        && document.value(left) == document.value(right)
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

#[cfg(test)]
mod tests {
    use super::{evaluate, parse};
    use crate::execution_control_experiment::InvocationControl;
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn document() -> Document {
        let parsed = parse_document(
            "urn:fastxslt:deep-equal:unit",
            br#"<doc><!--same--><a a="x"/><a a="x"/><b a="x"/><c c="x"/><!--same--><!--other--></doc>"#,
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("parse focused document");
        Document::from_parsed(parsed).expect("build focused document")
    }

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "urn:fastxslt:deep-equal:expression".to_owned(),
            span: 0..1,
        }
    }

    #[test]
    fn compares_values_and_expanded_names_without_using_node_identity() {
        let document = document();
        let mut control = InvocationControl::unbounded();
        let equal = parse("deep-equal(//a[1]/@a, //a[2]/@a)", &location())
            .expect("parse attribute equality");
        assert!(evaluate(&equal, &document, &mut control).expect("evaluate equal attributes"));

        let equal_value = parse("deep-equal(//a[1]/@a, //c[1]/@c)", &location())
            .expect("parse attribute comparison");
        assert!(
            !evaluate(&equal_value, &document, &mut control)
                .expect("compare equal values under different names")
        );

        let comments = parse("deep-equal(//comment()[1], //comment()[3])", &location())
            .expect("parse comment comparison");
        assert!(!evaluate(&comments, &document, &mut control).expect("evaluate comments"));
    }
}
