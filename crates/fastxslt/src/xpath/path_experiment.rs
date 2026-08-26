use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChildPath {
    pub(crate) steps: Vec<String>,
    pub(crate) selects_context_item: bool,
    pub(crate) starts_with_descendant_search: bool,
    final_predicate: Option<ExistencePredicate>,
    pub(crate) location: SourceLocation,
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
            location,
        });
    }
    let (expression, final_predicate) = parse_final_axis_predicate(expression);
    let (expression, starts_with_descendant_search) = expression
        .strip_prefix("//")
        .map_or((expression, false), |expression| (expression, true));
    if expression.starts_with('/')
        || expression.ends_with('/')
        || expression.contains("//")
        || expression.contains(['[', ']', '(', ')', '@', ':', '*'])
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

    let steps: Vec<_> = expression.split('/').map(str::to_owned).collect();
    if steps.iter().any(|step| step == "." || step == "..") {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice supports the context item only as the complete expression: {expression}"
            ),
            location,
        });
    }
    if steps.iter().any(|step| !is_ascii_ncname(step)) {
        return Err(PathFailure::Invalid {
            detail: format!("the private slice found an invalid child name in: {expression}"),
            location,
        });
    }
    Ok(ChildPath {
        steps,
        selects_context_item: false,
        starts_with_descendant_search,
        final_predicate,
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
            for child in candidates {
                if step_index != 0 || !path.starts_with_descendant_search {
                    control.charge(WorkDomain::XPathNodeVisit, 1)?;
                }
                let matches_name = document.kind(child) == NodeKind::Element
                    && document
                        .name(child)
                        .is_some_and(|name| name.namespace.is_none() && name.local == *step);
                let matches_predicate = matches_name
                    && if step_index + 1 == path.steps.len()
                        && let Some(predicate) = &path.final_predicate
                    {
                        match predicate.axis {
                            PredicateAxis::Child => {
                                has_named_child(document, child, &predicate.name, control)?
                            }
                            PredicateAxis::Attribute => {
                                has_named_attribute(document, child, &predicate.name, control)?
                            }
                            PredicateAxis::Ancestor => has_named_ancestor(
                                document,
                                child,
                                &predicate.name,
                                false,
                                control,
                            )?,
                            PredicateAxis::AncestorOrSelf => {
                                has_named_ancestor(document, child, &predicate.name, true, control)?
                            }
                            PredicateAxis::DescendantOrSelf => has_named_descendant_or_self(
                                document,
                                child,
                                &predicate.name,
                                control,
                            )?,
                            PredicateAxis::Parent => {
                                has_named_parent(document, child, &predicate.name, control)?
                            }
                        }
                    } else {
                        true
                    };
                if matches_predicate {
                    next.push(child);
                }
            }
        }
        current = next;
    }
    Ok(current)
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
        ExistencePredicate, PathFailure, PredicateAxis, evaluate_child_path,
        evaluate_child_path_controlled, parse_child_path,
    };
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xdm::owned_tree_experiment::SourceLocation;
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
