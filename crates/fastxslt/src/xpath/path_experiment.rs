use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xpath::constant_integer_experiment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChildPath {
    pub(crate) steps: Vec<ChildStep>,
    pub(crate) selects_context_item: bool,
    pub(crate) starts_with_descendant_search: bool,
    final_predicate: Option<ExistencePredicate>,
    step_position_predicates: Vec<Option<PositionPredicate>>,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChildStep {
    Named(String),
    AnyElement,
    AnyNode,
}

impl ChildStep {
    fn from_validated(value: String) -> Self {
        match value.as_str() {
            "*" => Self::AnyElement,
            "node()" => Self::AnyNode,
            _ => Self::Named(value),
        }
    }
}

impl PartialEq<&str> for ChildStep {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::Named(value) => value == *other,
            Self::AnyElement => *other == "*",
            Self::AnyNode => *other == "node()",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PredicateAxis {
    Child,
    Attribute,
    Ancestor,
    AncestorOrSelf,
    DescendantOrSelf,
    Parent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExistencePredicate {
    axis: PredicateAxis,
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionPredicate {
    Select(usize),
    Last,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathFailure {
    Invalid {
        detail: String,
        location: SourceLocation,
    },
    Unsupported {
        detail: String,
        location: SourceLocation,
    },
}

pub(crate) fn parse_child_path(
    expression: &str,
    location: SourceLocation,
) -> Result<ChildPath, PathFailure> {
    if expression.is_empty() {
        return Err(PathFailure::Invalid {
            detail: "the path expression is empty".to_owned(),
            location,
        });
    }
    if expression == "." {
        return Ok(ChildPath {
            steps: Vec::new(),
            selects_context_item: true,
            starts_with_descendant_search: false,
            final_predicate: None,
            step_position_predicates: Vec::new(),
            location,
        });
    }
    let (expression, final_predicate) = parse_final_axis_predicate(expression);
    let (expression, starts_with_descendant_search) = expression
        .strip_prefix("//")
        .map_or((expression, false), |expression| (expression, true));
    let parsed_steps = if final_predicate.is_none() {
        parse_position_steps(expression)
    } else {
        Some((
            expression.split('/').map(str::to_owned).collect(),
            vec![None; expression.split('/').count()],
        ))
    };
    if expression.starts_with('/')
        || expression.ends_with('/')
        || expression.contains("//")
        || parsed_steps.is_none()
    {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice supports only relative unprefixed child-name paths: {expression}"
            ),
            location,
        });
    }

    if !expression.is_ascii() {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice does not yet classify or evaluate non-ASCII child names: {expression}"
            ),
            location,
        });
    }

    let (steps, step_position_predicates) = parsed_steps.expect("checked above");
    if steps.iter().any(|step| step == "." || step == "..") {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice supports the context item only as the complete expression: {expression}"
            ),
            location,
        });
    }
    if steps
        .iter()
        .any(|step| !matches!(step.as_str(), "*" | "node()") && !is_ascii_ncname(step))
    {
        if steps.iter().any(|step| {
            step.chars().any(|character| {
                !character.is_ascii_alphanumeric() && !matches!(character, '_' | '-' | '.')
            })
        }) {
            return Err(PathFailure::Unsupported {
                detail: format!(
                    "the expression uses syntax outside the private child-path grammar: {expression}"
                ),
                location,
            });
        }
        return Err(PathFailure::Invalid {
            detail: format!("the private slice found an invalid child name in: {expression}"),
            location,
        });
    }
    Ok(ChildPath {
        steps: steps.into_iter().map(ChildStep::from_validated).collect(),
        selects_context_item: false,
        starts_with_descendant_search,
        final_predicate,
        step_position_predicates,
        location,
    })
}

fn parse_final_axis_predicate(expression: &str) -> (&str, Option<ExistencePredicate>) {
    let Some((path, predicate)) = expression.split_once('[') else {
        return (expression, None);
    };
    let Some(predicate) = predicate.strip_suffix(']') else {
        return (expression, None);
    };
    let (axis, name) = if let Some(name) = predicate.strip_prefix("child::") {
        (PredicateAxis::Child, name)
    } else if let Some(name) = predicate.strip_prefix("attribute::") {
        (PredicateAxis::Attribute, name)
    } else if let Some(name) = predicate.strip_prefix("ancestor::") {
        (PredicateAxis::Ancestor, name)
    } else if let Some(name) = predicate.strip_prefix("ancestor-or-self::") {
        (PredicateAxis::AncestorOrSelf, name)
    } else if let Some(name) = predicate.strip_prefix("descendant-or-self::") {
        (PredicateAxis::DescendantOrSelf, name)
    } else if let Some(name) = predicate.strip_prefix("parent::") {
        (PredicateAxis::Parent, name)
    } else {
        return (expression, None);
    };
    if path.is_empty() || path.contains('[') || !is_ascii_ncname(name) {
        return (expression, None);
    }
    (
        path,
        Some(ExistencePredicate {
            axis,
            name: name.to_owned(),
        }),
    )
}

fn parse_position_steps(expression: &str) -> Option<(Vec<String>, Vec<Option<PositionPredicate>>)> {
    let raw_steps = split_path_steps(expression)?;
    let mut steps = Vec::with_capacity(raw_steps.len());
    let mut predicates = Vec::with_capacity(raw_steps.len());
    for raw_step in raw_steps {
        let (name, predicate) = if let Some((name, predicate)) = raw_step.split_once('[') {
            let predicate = predicate.strip_suffix(']')?;
            if name.is_empty() || predicate.contains(['[', ']']) {
                return None;
            }
            let predicate = if predicate.trim() == "last()" {
                PositionPredicate::Last
            } else {
                let value = constant_integer_experiment::evaluate(predicate).ok()?;
                usize::try_from(value)
                    .ok()
                    .filter(|position| *position > 0)
                    .map_or(PositionPredicate::Never, PositionPredicate::Select)
            };
            (name, Some(predicate))
        } else {
            (raw_step, None)
        };
        let name = name.strip_prefix("child::").unwrap_or(name);
        if name.is_empty() {
            return None;
        }
        steps.push(name.to_owned());
        predicates.push(predicate);
    }
    Some((steps, predicates))
}

fn split_path_steps(expression: &str) -> Option<Vec<&str>> {
    let mut steps = Vec::new();
    let mut start = 0;
    let mut bracket_depth: usize = 0;
    for (offset, character) in expression.char_indices() {
        match character {
            '[' => bracket_depth += 1,
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
            }
            '/' if bracket_depth == 0 => {
                steps.push(&expression[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    if bracket_depth != 0 {
        return None;
    }
    steps.push(&expression[start..]);
    Some(steps)
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

#[cfg(test)]
pub(crate) fn evaluate_child_path(
    document: &Document,
    context: NodeId,
    path: &ChildPath,
) -> Vec<NodeId> {
    let mut control = InvocationControl::unbounded();
    evaluate_child_path_controlled(document, context, path, &mut control)
        .expect("unbounded private control cannot reject XPath work")
}

pub(crate) fn evaluate_child_path_controlled(
    document: &Document,
    context: NodeId,
    path: &ChildPath,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    if path.selects_context_item {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        return Ok(vec![context]);
    }
    let mut current = vec![context];
    for (step_index, step) in path.steps.iter().enumerate() {
        let mut next = Vec::new();
        for node in current {
            let candidates = if step_index == 0 && path.starts_with_descendant_search {
                descendant_nodes(document, node, control)?
            } else {
                document.children(node).to_vec()
            };
            let mut named_candidates = Vec::new();
            for child in candidates {
                if step_index != 0 || !path.starts_with_descendant_search {
                    control.charge(WorkDomain::XPathNodeVisit, 1)?;
                }
                if child_matches_name_test(document, child, step) {
                    named_candidates.push(child);
                }
            }
            let matching_count = named_candidates.len();
            for (offset, child) in named_candidates.into_iter().enumerate() {
                let position_matches = match path.step_position_predicates[step_index] {
                    Some(PositionPredicate::Select(position)) => position == offset + 1,
                    Some(PositionPredicate::Last) => offset + 1 == matching_count,
                    Some(PositionPredicate::Never) => false,
                    None => true,
                };
                let existence_matches = if step_index + 1 == path.steps.len()
                    && let Some(predicate) = &path.final_predicate
                {
                    match predicate.axis {
                        PredicateAxis::Child => {
                            has_named_child(document, child, &predicate.name, control)?
                        }
                        PredicateAxis::Attribute => {
                            has_named_attribute(document, child, &predicate.name, control)?
                        }
                        PredicateAxis::Ancestor => {
                            has_named_ancestor(document, child, &predicate.name, false, control)?
                        }
                        PredicateAxis::AncestorOrSelf => {
                            has_named_ancestor(document, child, &predicate.name, true, control)?
                        }
                        PredicateAxis::DescendantOrSelf => {
                            has_named_descendant_or_self(document, child, &predicate.name, control)?
                        }
                        PredicateAxis::Parent => {
                            has_named_parent(document, child, &predicate.name, control)?
                        }
                    }
                } else {
                    true
                };
                if position_matches && existence_matches {
                    next.push(child);
                }
            }
        }
        current = next;
    }
    Ok(current)
}

fn child_matches_name_test(document: &Document, child: NodeId, name_test: &ChildStep) -> bool {
    match name_test {
        ChildStep::Named(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        ChildStep::AnyElement => document.kind(child) == NodeKind::Element,
        ChildStep::AnyNode => true,
    }
}

fn descendant_nodes(
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let mut descendants = Vec::new();
    let mut pending: Vec<_> = document.children(context).iter().rev().copied().collect();
    while let Some(node) = pending.pop() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        pending.extend(document.children(node).iter().rev().copied());
        descendants.push(node);
    }
    Ok(descendants)
}

fn has_named_child(
    document: &Document,
    node: NodeId,
    required: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for child in document.children(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if node_has_unnamespaced_name(document, child, required) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn has_named_attribute(
    document: &Document,
    node: NodeId,
    required: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for attribute in document.attributes(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(attribute) == NodeKind::Attribute
            && document
                .name(attribute)
                .is_some_and(|name| name.namespace.is_none() && name.local == required)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn has_named_descendant_or_self(
    document: &Document,
    node: NodeId,
    required: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut pending = vec![node];
    while let Some(candidate) = pending.pop() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if node_has_unnamespaced_name(document, candidate, required) {
            return Ok(true);
        }
        pending.extend(document.children(candidate).iter().rev().copied());
    }
    Ok(false)
}

fn has_named_parent(
    document: &Document,
    node: NodeId,
    required: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let Some(parent) = document.parent(node) else {
        return Ok(false);
    };
    control.charge(WorkDomain::XPathNodeVisit, 1)?;
    Ok(node_has_unnamespaced_name(document, parent, required))
}

fn has_named_ancestor(
    document: &Document,
    node: NodeId,
    required: &str,
    include_self: bool,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut ancestor = include_self
        .then_some(node)
        .or_else(|| document.parent(node));
    while let Some(node) = ancestor {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if node_has_unnamespaced_name(document, node, required) {
            return Ok(true);
        }
        ancestor = document.parent(node);
    }
    Ok(false)
}

fn node_has_unnamespaced_name(document: &Document, node: NodeId, required: &str) -> bool {
    document.kind(node) == NodeKind::Element
        && document
            .name(node)
            .is_some_and(|name| name.namespace.is_none() && name.local == required)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::{
        ExistencePredicate, PathFailure, PositionPredicate, PredicateAxis, evaluate_child_path,
        evaluate_child_path_controlled, parse_child_path,
    };
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::{Document, NodeKind, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "memory:stylesheet.xsl".to_owned(),
            span: Range { start: 12, end: 25 },
        }
    }

    #[test]
    fn parses_the_golden_relative_child_path() {
        let path = parse_child_path("greeting/name", location()).expect("path should parse");

        assert_eq!(path.steps, ["greeting", "name"]);
        assert!(!path.selects_context_item);
        assert_eq!(path.final_predicate, None);
        assert!(!path.starts_with_descendant_search);
        assert_eq!(path.location, location());
    }

    #[test]
    fn selects_the_context_item_without_navigation() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<item>value</item>",
            ParseLimits {
                max_events: 8,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let item = document.children(document.document_node())[0];
        let path = parse_child_path(".", location()).expect("context item should parse");

        let selected = evaluate_child_path(&document, item, &path);

        assert_eq!(selected, [item]);
        assert!(path.selects_context_item);
    }

    #[test]
    fn distinguishes_invalid_from_unsupported_path_syntax() {
        assert!(matches!(
            parse_child_path("", location()),
            Err(PathFailure::Invalid { .. })
        ));
        let invalid = parse_child_path("1greeting/name", location())
            .expect_err("an invalid child-name path must fail");
        let unsupported = parse_child_path("greeting//name", location())
            .expect_err("a descendant path is outside the private slice");

        assert!(matches!(invalid, PathFailure::Invalid { .. }));
        assert!(matches!(unsupported, PathFailure::Unsupported { .. }));
        assert_eq!(failure_location(&invalid), &location());
        assert_eq!(failure_location(&unsupported), &location());
        assert!(matches!(
            parse_child_path("sum(for $i in item return $i)", location()),
            Err(PathFailure::Unsupported { .. })
        ));
    }

    #[test]
    fn accepts_supported_ncname_punctuation_without_claiming_unicode_names() {
        let path = parse_child_path("catalog/item.name/item-2/_value", location())
            .expect("ASCII NCName punctuation belongs to the private grammar");
        assert_eq!(path.steps, ["catalog", "item.name", "item-2", "_value"]);

        assert!(matches!(
            parse_child_path("café/name", location()),
            Err(PathFailure::Unsupported { .. })
        ));
        assert!(matches!(
            parse_child_path("catalog/..", location()),
            Err(PathFailure::Unsupported { .. })
        ));
    }

    #[test]
    fn explicit_named_child_axis_uses_the_same_child_navigation_semantics() {
        let implicit = parse_child_path("//center/south-east", location())
            .expect("implicit child steps should parse");
        let explicit = parse_child_path("//center/child::south-east", location())
            .expect("explicit named child-axis step should parse");

        assert_eq!(explicit.steps, implicit.steps);
        assert!(explicit.starts_with_descendant_search);
    }

    #[test]
    fn explicit_child_wildcard_selects_elements_across_namespaces() {
        let parsed = parse_document(
            "memory:source.xml",
            br#"<root>text<a/><n:b xmlns:n="urn:test"/><!-- comment --></root>"#,
            ParseLimits {
                max_events: 16,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let root = document.children(document.document_node())[0];
        let path = parse_child_path("child::*", location()).expect("child wildcard should parse");
        let mut control = InvocationControl::unbounded();

        let selected = evaluate_child_path_controlled(&document, root, &path, &mut control)
            .expect("unbounded evaluation should succeed");

        assert_eq!(selected.len(), 2);
        assert_eq!(document.name(selected[0]).expect("a name").local, "a");
        assert_eq!(document.name(selected[1]).expect("b name").local, "b");
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 4);
    }

    #[test]
    fn explicit_child_node_test_selects_every_child_node_kind() {
        let parsed = parse_document(
            "memory:source.xml",
            br"<root>text<a/><?work item?><!-- comment --></root>",
            ParseLimits {
                max_events: 16,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let root = document.children(document.document_node())[0];
        let path =
            parse_child_path("child::node()", location()).expect("child node test should parse");
        let mut control = InvocationControl::unbounded();

        let selected = evaluate_child_path_controlled(&document, root, &path, &mut control)
            .expect("unbounded evaluation should succeed");

        assert_eq!(
            selected
                .iter()
                .map(|node| document.kind(*node))
                .collect::<Vec<_>>(),
            [
                NodeKind::Text,
                NodeKind::Element,
                NodeKind::ProcessingInstruction,
                NodeKind::Comment,
            ]
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 4);
    }

    #[test]
    fn evaluates_the_golden_path_from_the_document_node() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<greeting><name>FastXSLT</name></greeting>",
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let path = parse_child_path("greeting/name", location()).expect("path should parse");

        let selected = evaluate_child_path(&document, document.document_node(), &path);

        assert_eq!(selected.len(), 1);
        assert_eq!(document.string_value(selected[0]), "FastXSLT");
    }

    #[test]
    fn filters_the_final_child_step_by_an_explicit_named_child_axis() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<doc><child1/><child1><child2/></child1><child1><other/></child1></doc>",
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let doc = document.children(document.document_node())[0];
        let path = parse_child_path("child1[child::child2]", location())
            .expect("named child-axis predicate should parse");

        let mut control = InvocationControl::unbounded();
        let selected = evaluate_child_path_controlled(&document, doc, &path, &mut control)
            .expect("unbounded evaluation should succeed");

        assert_eq!(selected.len(), 1);
        assert_eq!(
            path.final_predicate,
            Some(ExistencePredicate {
                axis: PredicateAxis::Child,
                name: "child2".to_owned(),
            })
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 5);
    }

    #[test]
    fn searches_descendants_and_filters_by_a_named_ancestor() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<doc><element1><child2>wrong</child2></element1><element2><child2>right</child2></element2></doc>",
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let doc = document.children(document.document_node())[0];
        let path = parse_child_path("//child2[ancestor::element2]", location())
            .expect("path-002 expression should parse");

        let mut control = InvocationControl::unbounded();
        let selected = evaluate_child_path_controlled(&document, doc, &path, &mut control)
            .expect("unbounded evaluation should succeed");

        assert!(path.starts_with_descendant_search);
        assert_eq!(selected.len(), 1);
        assert_eq!(document.string_value(selected[0]), "right");
        assert_eq!(
            path.final_predicate,
            Some(ExistencePredicate {
                axis: PredicateAxis::Ancestor,
                name: "element2".to_owned(),
            })
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 10);
    }

    #[test]
    fn ancestor_or_self_predicate_checks_the_candidate_before_its_parent() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<doc><element2><child2>right</child2></element2></doc>",
            ParseLimits {
                max_events: 24,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let doc = document.children(document.document_node())[0];
        let self_path = parse_child_path("//element2[ancestor-or-self::element2]", location())
            .expect("ancestor-or-self self match should parse");
        let ancestor_path = parse_child_path("//child2[ancestor-or-self::element2]", location())
            .expect("path-003 expression should parse");

        let self_selected = evaluate_child_path(&document, doc, &self_path);
        let ancestor_selected = evaluate_child_path(&document, doc, &ancestor_path);

        assert_eq!(self_selected.len(), 1);
        assert_eq!(document.string_value(ancestor_selected[0]), "right");
        assert_eq!(
            self_path.final_predicate,
            Some(ExistencePredicate {
                axis: PredicateAxis::AncestorOrSelf,
                name: "element2".to_owned(),
            })
        );
    }

    #[test]
    fn attribute_predicate_inspects_attributes_without_making_them_children() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<doc><child2/><child2 attr1=\"yes\">right</child2></doc>",
            ParseLimits {
                max_events: 24,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let doc = document.children(document.document_node())[0];
        let path = parse_child_path("//child2[attribute::attr1]", location())
            .expect("path-004 expression should parse");

        let mut control = InvocationControl::unbounded();
        let selected = evaluate_child_path_controlled(&document, doc, &path, &mut control)
            .expect("unbounded evaluation should succeed");

        assert_eq!(selected.len(), 1);
        assert_eq!(document.string_value(selected[0]), "right");
        assert_eq!(document.children(selected[0]).len(), 1);
        assert_eq!(document.attributes(selected[0]).len(), 1);
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 4);
        assert_eq!(
            path.final_predicate,
            Some(ExistencePredicate {
                axis: PredicateAxis::Attribute,
                name: "attr1".to_owned(),
            })
        );
    }

    #[test]
    fn descendant_or_self_predicate_checks_self_then_document_order_descendants() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<doc><element1><child2>right</child2></element1><element1><child1/></element1><child2>self</child2></doc>",
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let doc = document.children(document.document_node())[0];
        let descendant_path = parse_child_path("element1[descendant-or-self::child2]", location())
            .expect("path-005 expression should parse");
        let self_path = parse_child_path("child2[descendant-or-self::child2]", location())
            .expect("descendant-or-self self match should parse");
        let mut control = InvocationControl::unbounded();

        let descendant_selected =
            evaluate_child_path_controlled(&document, doc, &descendant_path, &mut control)
                .expect("unbounded evaluation should succeed");
        let self_selected = evaluate_child_path(&document, doc, &self_path);

        assert_eq!(descendant_selected.len(), 1);
        assert_eq!(document.string_value(descendant_selected[0]), "right");
        assert_eq!(document.string_value(self_selected[0]), "self");
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 7);
        assert_eq!(
            descendant_path.final_predicate,
            Some(ExistencePredicate {
                axis: PredicateAxis::DescendantOrSelf,
                name: "child2".to_owned(),
            })
        );
    }

    #[test]
    fn parent_predicate_checks_only_the_immediate_parent() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<doc><element1><child1>right</child1></element1><element2><child1>wrong</child1></element2></doc>",
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let doc = document.children(document.document_node())[0];
        let path = parse_child_path("//child1[parent::element1]", location())
            .expect("path-006 expression should parse");
        let mut control = InvocationControl::unbounded();

        let selected = evaluate_child_path_controlled(&document, doc, &path, &mut control)
            .expect("unbounded evaluation should succeed");

        assert_eq!(selected.len(), 1);
        assert_eq!(document.string_value(selected[0]), "right");
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 8);
        assert_eq!(
            path.final_predicate,
            Some(ExistencePredicate {
                axis: PredicateAxis::Parent,
                name: "element1".to_owned(),
            })
        );
    }

    #[test]
    fn constant_integer_arithmetic_selects_the_matching_node_position() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<doc><element1>wrong</element1><skip/><element1>right</element1><element1>wrong</element1></doc>",
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let doc = document.children(document.document_node())[0];
        let path = parse_child_path("element1[(((((2*10)-4)+9) div 5) mod 3 )]", location())
            .expect("path-007 expression should parse");
        let mut control = InvocationControl::unbounded();

        let selected = evaluate_child_path_controlled(&document, doc, &path, &mut control)
            .expect("unbounded evaluation should succeed");

        assert_eq!(selected.len(), 1);
        assert_eq!(document.string_value(selected[0]), "right");
        assert_eq!(
            path.step_position_predicates[0],
            Some(PositionPredicate::Select(2))
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 4);
    }

    #[test]
    fn applies_positions_to_individual_steps_and_last_to_the_matched_sequence() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<doc><element1>wrong</element1><element1><child1>wrong</child1><child1>wrong</child1><child1>right</child1></element1><element1>wrong</element1></doc>",
            ParseLimits {
                max_events: 40,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let path = parse_child_path(
            "doc/element1[(((((2*10)-4)+9) div 5) mod 3)]/child1[last()]",
            location(),
        )
        .expect("path-010 selection should parse");
        let mut control = InvocationControl::unbounded();

        let selected = evaluate_child_path_controlled(
            &document,
            document.document_node(),
            &path,
            &mut control,
        )
        .expect("unbounded evaluation should succeed");

        assert_eq!(selected.len(), 1);
        assert_eq!(document.string_value(selected[0]), "right");
        assert_eq!(
            path.step_position_predicates,
            [
                None,
                Some(PositionPredicate::Select(2)),
                Some(PositionPredicate::Last),
            ]
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 7);
    }

    #[test]
    fn evaluation_preserves_document_order_and_requires_no_namespace() {
        let parsed = parse_document(
            "memory:source.xml",
            br#"<catalog xmlns:n="urn:other"><item>first</item><n:item>namespaced</n:item><skip/><item>second</item><item.name>dotted</item.name></catalog>"#,
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let catalog = document.children(document.document_node())[0];
        let items = parse_child_path("item", location()).expect("item path should parse");
        let dotted = parse_child_path("item.name", location()).expect("dotted name should parse");
        let missing = parse_child_path("missing", location()).expect("missing path should parse");

        let selected = evaluate_child_path(&document, catalog, &items);

        assert_eq!(selected.len(), 2);
        assert_eq!(document.string_value(selected[0]), "first");
        assert_eq!(document.string_value(selected[1]), "second");
        assert_eq!(
            document.string_value(evaluate_child_path(&document, catalog, &dotted)[0]),
            "dotted"
        );
        assert!(evaluate_child_path(&document, catalog, &missing).is_empty());
    }

    fn failure_location(failure: &PathFailure) -> &SourceLocation {
        match failure {
            PathFailure::Invalid { location, .. } | PathFailure::Unsupported { location, .. } => {
                location
            }
        }
    }
}
