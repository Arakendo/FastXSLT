use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeepEqualExpression {
    operands: DeepEqualOperands,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeepEqualOperands {
    Nodes {
        left: NodeSelection,
        right: NodeSelection,
    },
    Integers {
        left: i128,
        right: i128,
    },
    Decimals {
        left: ExactDecimal,
        right: ExactDecimal,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExactDecimal {
    coefficient: i128,
    scale: u32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeepEqualEvaluationFailure {
    Control(ControlFailure),
    MissingNodeContext,
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<DeepEqualExpression, DeepEqualFailure> {
    let arguments = expression
        .strip_prefix("deep-equal(")
        .or_else(|| expression.strip_prefix("fn:deep-equal("))
        .and_then(|value| value.strip_suffix(')'))
        .and_then(|value| value.split_once(','))
        .ok_or_else(|| unsupported(expression, location))?;
    let left = arguments.0.trim();
    let right = arguments.1.trim();
    let operands = if let (Some(left), Some(right)) = (parse_integer(left), parse_integer(right)) {
        DeepEqualOperands::Integers { left, right }
    } else if let (Some(left), Some(right)) = (parse_decimal(left), parse_decimal(right)) {
        DeepEqualOperands::Decimals { left, right }
    } else {
        DeepEqualOperands::Nodes {
            left: parse_selection(left, location)?,
            right: parse_selection(right, location)?,
        }
    };
    Ok(DeepEqualExpression {
        operands,
        location: location.clone(),
    })
}

fn parse_decimal(expression: &str) -> Option<ExactDecimal> {
    let lexical = expression
        .strip_prefix("(xs:decimal(\"")
        .and_then(|value| value.strip_suffix("\"))"))?;
    let (negative, magnitude) = lexical
        .strip_prefix('-')
        .map_or((false, lexical), |value| (true, value));
    let (integer, fraction) = magnitude.split_once('.').unwrap_or((magnitude, ""));
    if integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let mut scale = u32::try_from(fraction.len()).ok()?;
    let digits = format!("{integer}{fraction}");
    let mut coefficient = digits.parse::<i128>().ok()?;
    while scale > 0 && coefficient % 10 == 0 {
        coefficient /= 10;
        scale -= 1;
    }
    if negative {
        coefficient = coefficient.checked_neg()?;
    }
    Some(ExactDecimal { coefficient, scale })
}

fn parse_integer(expression: &str) -> Option<i128> {
    let int = expression
        .strip_prefix("(xs:int(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i32>().ok())
        .map(i128::from);
    let long = expression
        .strip_prefix("(xs:long(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i64>().ok())
        .map(i128::from);
    int.or(long).or_else(|| {
        expression
            .strip_prefix("(xs:integer(\"")
            .and_then(|value| value.strip_suffix("\"))"))
            .and_then(|value| value.parse::<i128>().ok())
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
    document: Option<&Document>,
    control: &mut InvocationControl,
) -> Result<bool, DeepEqualEvaluationFailure> {
    match &expression.operands {
        DeepEqualOperands::Integers { left, right } => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(DeepEqualEvaluationFailure::Control)?;
            Ok(left == right)
        }
        DeepEqualOperands::Decimals { left, right } => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(DeepEqualEvaluationFailure::Control)?;
            Ok(left == right)
        }
        DeepEqualOperands::Nodes { left, right } => {
            let document = document.ok_or(DeepEqualEvaluationFailure::MissingNodeContext)?;
            evaluate_nodes(left, right, document, control)
        }
    }
}

fn evaluate_nodes(
    left: &NodeSelection,
    right: &NodeSelection,
    document: &Document,
    control: &mut InvocationControl,
) -> Result<bool, DeepEqualEvaluationFailure> {
    let left =
        select_nodes(left, document, control).map_err(DeepEqualEvaluationFailure::Control)?;
    let right =
        select_nodes(right, document, control).map_err(DeepEqualEvaluationFailure::Control)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.into_iter().zip(right) {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(DeepEqualEvaluationFailure::Control)?;
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
        assert!(
            evaluate(&equal, Some(&document), &mut control).expect("evaluate equal attributes")
        );

        let equal_value = parse("deep-equal(//a[1]/@a, //c[1]/@c)", &location())
            .expect("parse attribute comparison");
        assert!(
            !evaluate(&equal_value, Some(&document), &mut control)
                .expect("compare equal values under different names")
        );

        let comments = parse("deep-equal(//comment()[1], //comment()[3])", &location())
            .expect("parse comment comparison");
        assert!(!evaluate(&comments, Some(&document), &mut control).expect("evaluate comments"));
    }

    #[test]
    fn compares_qt3_xs_int_values_numerically() {
        let mut control = InvocationControl::unbounded();
        let equal = parse(
            "fn:deep-equal((xs:int(\"-2147483648\")),(xs:int(\"-2147483648\")))",
            &location(),
        )
        .expect("parse typed integer equality");
        assert!(evaluate(&equal, None, &mut control).expect("evaluate typed integers"));
    }

    #[test]
    fn normalizes_exact_decimal_scale_without_binary_floating_point() {
        let mut control = InvocationControl::unbounded();
        let equal = parse(
            "fn:deep-equal((xs:decimal(\"1.0\")),(xs:decimal(\"1.00\")))",
            &location(),
        )
        .expect("parse exact decimal equality");
        assert!(evaluate(&equal, None, &mut control).expect("evaluate exact decimals"));
    }

    #[test]
    fn enforces_the_xs_long_value_range() {
        let in_range = parse(
            "fn:deep-equal((xs:long(\"9223372036854775807\")),(xs:long(\"9223372036854775807\")))",
            &location(),
        )
        .expect("parse maximum xs:long equality");
        assert!(
            evaluate(&in_range, None, &mut InvocationControl::unbounded())
                .expect("evaluate xs:long equality")
        );
        assert!(
            parse(
                "fn:deep-equal((xs:long(\"9223372036854775808\")),(xs:long(\"9223372036854775808\")))",
                &location(),
            )
            .is_err()
        );
    }
}
