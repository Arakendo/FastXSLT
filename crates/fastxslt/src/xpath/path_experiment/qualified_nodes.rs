//! Qualified source path results over the existing tree evaluator (ADR-0021).
//! Terminal namespace selection admits charged scalar-literal or integer filters.

use super::{
    LocationPath, PathFailure, evaluate_location_path_controlled, is_ncname, parse_location_path,
};
use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xdm::qualified_nodes::{NamespaceFailure, QualifiedSourceNode};

#[path = "namespace_predicate.rs"]
mod namespace_predicate;
use namespace_predicate::NamespacePredicate;

#[cfg(test)]
#[path = "qualified_union_tests.rs"]
mod qualified_union_tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QualifiedLocationPath {
    Tree(LocationPath),
    Union(Vec<QualifiedLocationPath>),
    Namespace {
        owners: LocationPath,
        prefix: Option<String>,
        predicate: Option<NamespacePredicate>,
    },
}

impl QualifiedLocationPath {
    pub(crate) fn location(&self) -> &SourceLocation {
        match self {
            Self::Tree(path) => &path.location,
            Self::Union(paths) => paths[0].location(),
            Self::Namespace { owners, .. } => &owners.location,
        }
    }
    #[cfg(feature = "workbench")]
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Tree(path) => path.known_owned_capacity_bytes(),
            Self::Union(paths) => {
                paths.capacity() * std::mem::size_of::<Self>()
                    + paths
                        .iter()
                        .map(Self::known_owned_capacity_bytes)
                        .sum::<usize>()
            }
            Self::Namespace {
                owners,
                prefix,
                predicate,
            } => {
                owners.known_owned_capacity_bytes()
                    + prefix.as_ref().map_or(0, String::capacity)
                    + predicate
                        .as_ref()
                        .map_or(0, NamespacePredicate::known_owned_capacity_bytes)
            }
        }
    }
}

pub(crate) fn parse(
    expression: &str,
    location: SourceLocation,
) -> Result<QualifiedLocationPath, PathFailure> {
    let expression = expression.trim();
    if !recognizes_namespace_axis(expression) {
        return parse_location_path(expression, location).map(QualifiedLocationPath::Tree);
    }
    if let Some(alternatives) = union_alternatives(expression) {
        let mut paths = Vec::new();
        for alternative in alternatives {
            if alternative.trim().is_empty() {
                return Err(PathFailure::Invalid {
                    standard_code: "XPST0003",
                    detail: "qualified union contains an empty alternative".into(),
                    location,
                });
            }
            paths.push(parse(alternative, location.clone())?);
        }
        return Ok(QualifiedLocationPath::Union(paths));
    }
    // A literal predicate value may contain '/'; it is not a path separator.
    let (owner, terminal) = expression
        .split_once("/namespace::")
        .map_or((".", expression), |(owner, terminal)| (owner, terminal));
    let terminal = if expression.contains("/namespace::") {
        terminal
    } else {
        terminal.strip_prefix("namespace::").unwrap_or(terminal)
    };
    if !expression.starts_with("namespace::") && !expression.contains("/namespace::") {
        return Err(PathFailure::Unsupported {
            detail: "namespace selection is only admitted as a terminal path step".into(),
            location,
        });
    }
    let (test, predicate) =
        namespace_predicate::parse_terminal(terminal).ok_or_else(|| PathFailure::Unsupported {
            detail: "this namespace predicate or terminal composition is not admitted".into(),
            location: location.clone(),
        })?;
    let prefix = if matches!(test, "*" | "node()") {
        None
    } else if is_ncname(test) {
        Some(test.to_owned())
    } else {
        return Err(PathFailure::Unsupported {
            detail: "compound namespace name tests are not admitted".into(),
            location,
        });
    };
    // Preserve // instead of converting a descendant separator to a child step.
    if owner.ends_with('/') || owner.is_empty() || owner.contains("namespace::") {
        return Err(PathFailure::Unsupported {
            detail: "this namespace owner path is not admitted".into(),
            location,
        });
    }
    let owners = parse_location_path(owner, location)?;
    Ok(QualifiedLocationPath::Namespace {
        owners,
        prefix,
        predicate,
    })
}

pub(crate) fn recognizes_namespace_axis(expression: &str) -> bool {
    let mut quote = None;
    for (index, character) in expression.char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
        } else if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if expression[index..].starts_with("namespace::") {
            return true;
        }
    }
    false
}

fn union_alternatives(expression: &str) -> Option<Vec<&str>> {
    let mut quote = None;
    let mut depth = 0_usize;
    let mut start = 0;
    let mut alternatives = Vec::new();
    for (index, character) in expression.char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.checked_sub(1)?,
            '|' if depth == 0 => {
                alternatives.push(&expression[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if quote.is_some() || depth != 0 || alternatives.is_empty() {
        return None;
    }
    alternatives.push(&expression[start..]);
    Some(alternatives)
}

/// One semantic parent-axis visit, without allocating or losing owner qualification.
pub(crate) fn parent_controlled<'a>(
    node: QualifiedSourceNode<'a>,
    control: &mut InvocationControl,
) -> Result<Option<QualifiedSourceNode<'a>>, ControlFailure> {
    control.charge(WorkDomain::XPathNodeVisit, 1)?;
    Ok(node.parent())
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum QualifiedPathFailure {
    Control(ControlFailure),
    Namespace(NamespaceFailure),
    SequenceCapacity,
}

pub(crate) fn evaluate<'a>(
    document: &'a Document,
    context: NodeId,
    path: &QualifiedLocationPath,
    max_bindings: usize,
    max_nodes: usize,
    control: &mut InvocationControl,
) -> Result<Vec<QualifiedSourceNode<'a>>, QualifiedPathFailure> {
    if let QualifiedLocationPath::Union(paths) = path {
        let mut result = Vec::new();
        for path in paths {
            for node in evaluate(document, context, path, max_bindings, max_nodes, control)? {
                push(&mut result, node, max_nodes, control)?;
            }
        }
        // Abstract normalization work, not a claim about physical comparator calls.
        let normalization_charge = result.len().saturating_mul(
            result
                .len()
                .checked_ilog2()
                .map_or(0, |value| value as usize + 1),
        );
        control
            .charge(WorkDomain::XPathNodeVisit, normalization_charge)
            .map_err(QualifiedPathFailure::Control)?;
        result.sort_unstable_by(|left, right| {
            left.compare_same_origin(*right)
                .expect("one effective source owns every union operand")
        });
        control
            .charge(WorkDomain::XPathNodeVisit, result.len())
            .map_err(QualifiedPathFailure::Control)?;
        result.dedup_by(|left, right| left.same_node(*right));
        return Ok(result);
    }
    let owners = match path {
        QualifiedLocationPath::Union(_) => unreachable!("union handled before ordinary selection"),
        QualifiedLocationPath::Tree(path) => path,
        QualifiedLocationPath::Namespace { owners, .. } => owners,
    };
    let owners = evaluate_location_path_controlled(document, context, owners, control)
        .map_err(QualifiedPathFailure::Control)?;
    let mut result = Vec::new();
    for owner in owners {
        match path {
            QualifiedLocationPath::Union(_) => {
                unreachable!("union handled before ordinary selection")
            }
            QualifiedLocationPath::Tree(_) => {
                push(
                    &mut result,
                    QualifiedSourceNode::Tree {
                        document,
                        id: owner,
                    },
                    max_nodes,
                    control,
                )?;
            }
            QualifiedLocationPath::Namespace {
                prefix, predicate, ..
            } => {
                let occurrences = document
                    .namespace_nodes_controlled(owner, max_bindings, control)
                    .map_err(QualifiedPathFailure::Namespace)?;
                let mut position = 0;
                for node in occurrences {
                    control
                        .charge(WorkDomain::XPathNodeVisit, 1)
                        .map_err(QualifiedPathFailure::Control)?;
                    let named = prefix.as_deref().is_none_or(|prefix| {
                        node.namespace().is_some_and(|node| node.prefix() == prefix)
                    });
                    let matches = if named {
                        position += 1;
                        match predicate {
                            Some(predicate) => predicate
                                .matches(node, position, control)
                                .map_err(QualifiedPathFailure::Control)?,
                            None => true,
                        }
                    } else {
                        false
                    };
                    if matches {
                        push(&mut result, node, max_nodes, control)?;
                    }
                }
            }
        }
    }
    // Existing owner paths are normalized. Each owner's namespaces are already
    // unique and prefix-ordered, so terminal composition needs no extra sort.
    Ok(result)
}

fn push<'a>(
    result: &mut Vec<QualifiedSourceNode<'a>>,
    node: QualifiedSourceNode<'a>,
    max_nodes: usize,
    control: &mut InvocationControl,
) -> Result<(), QualifiedPathFailure> {
    if result.len() == max_nodes {
        return Err(QualifiedPathFailure::SequenceCapacity);
    }
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(QualifiedPathFailure::Control)?;
    result.push(node);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn source() -> Document {
        Document::from_parsed(
            parse_document(
                "memory:qualified-path.xml",
                b"<r xmlns:p='urn:p'><a/><a/></r>",
                ParseLimits {
                    max_events: 20,
                    max_depth: 10,
                },
            )
            .unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn namespace_path_returns_qualified_occurrences_not_owner_ids() {
        let source = source();
        for expression in ["/r/a/namespace::*", "/r/a/namespace::node()"] {
            let path = parse(expression, source.location(source.document_node()).clone()).unwrap();
            let nodes = evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                8,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(nodes.len(), 4);
            assert_eq!(
                nodes
                    .iter()
                    .map(|node| node.namespace().unwrap().prefix())
                    .collect::<Vec<_>>(),
                ["p", "xml", "p", "xml"]
            );
            assert!(!nodes[0].same_node(nodes[2]));
            assert!(
                nodes[0]
                    .parent()
                    .unwrap()
                    .same_node(nodes[1].parent().unwrap())
            );
        }
        let path = parse(
            "/r/a/namespace::p",
            source.location(source.document_node()).clone(),
        )
        .unwrap();
        let nodes = evaluate(
            &source,
            source.document_node(),
            &path,
            8,
            8,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(
            nodes
                .iter()
                .all(|node| node.namespace().unwrap().string_value() == "urn:p")
        );
    }
    #[test]
    fn tree_only_paths_keep_existing_identity_and_unsupported_shapes_stay_closed() {
        let source = source();
        let location = source.location(source.document_node()).clone();
        let path = parse("/r/a", location.clone()).unwrap();
        let qualified = evaluate(
            &source,
            source.document_node(),
            &path,
            8,
            8,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let tree = evaluate_location_path_controlled(
            &source,
            source.document_node(),
            &parse_location_path("/r/a", location.clone()).unwrap(),
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(qualified.len(), tree.len());
        for (qualified, id) in qualified.into_iter().zip(tree) {
            assert!(qualified.same_node(QualifiedSourceNode::Tree {
                document: &source,
                id
            }));
        }
        for expression in [
            "namespace::*[position()=1]",
            "namespace::*[1.5]",
            "namespace::*[-1]",
            "namespace::*[name()='p'][1]",
            "namespace::*[name()='p' or name()='xml']",
            "namespace::*[string()='unterminated]",
            "namespace::p/..",
            "/r//namespace::*",
            "namespace::p:q",
            "namespace::*/namespace::*",
        ] {
            assert!(matches!(
                parse(expression, location.clone()),
                Err(PathFailure::Unsupported { .. })
            ));
        }
    }

    #[test]
    fn namespace_filters_are_bounded_and_literal_axis_text_is_not_syntax() {
        let source = source();
        let location = source.location(source.document_node()).clone();
        assert!(!recognizes_namespace_axis("concat('namespace::', 'x')"));
        assert!(!recognizes_namespace_axis("r[@a='namespace::*']"));
        for expression in [
            "/r/a/namespace::*[name(.)='p']",
            "/r/a/namespace::*[string()='urn:p']",
            "/r/a/namespace::*[local-name()!='xml']",
        ] {
            let path = parse(expression, location.clone()).unwrap();
            let nodes = evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                2,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(nodes.len(), 2);
            assert!(!nodes[0].same_node(nodes[1]));
            assert!(
                nodes
                    .iter()
                    .all(|node| node.namespace().unwrap().prefix() == "p")
            );
            assert_eq!(
                evaluate(
                    &source,
                    source.document_node(),
                    &path,
                    8,
                    1,
                    &mut InvocationControl::unbounded()
                )
                .unwrap_err(),
                QualifiedPathFailure::SequenceCapacity
            );
        }
    }

    #[test]
    fn integer_namespace_positions_match_complete_selection_and_discard_partial_results() {
        let source = source();
        let location = source.location(source.document_node()).clone();
        for (test, position) in [
            ("*", 1_usize),
            ("*", 2),
            ("p", 1),
            ("p", 2),
            ("xml", 1),
            ("*", 0),
        ] {
            let all = evaluate(
                &source,
                source.document_node(),
                &parse(&format!("/r/a/namespace::{test}"), location.clone()).unwrap(),
                8,
                8,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let path = parse(
                &format!("/r/a/namespace::{test}[{position}]"),
                location.clone(),
            )
            .unwrap();
            let selected = evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                8,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let per_owner = if test == "*" { 2 } else { 1 };
            let expected: Vec<_> = all
                .chunks(per_owner)
                .filter_map(|nodes| position.checked_sub(1).and_then(|index| nodes.get(index)))
                .collect();
            assert_eq!(selected.len(), expected.len());
            for (actual, expected) in selected.iter().zip(expected) {
                assert!(actual.same_node(*expected));
            }
        }
        let path = parse("/r/a/namespace::*[1]", location).unwrap();
        assert_eq!(
            evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                1,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err(),
            QualifiedPathFailure::SequenceCapacity
        );
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 2);
        assert_eq!(
            evaluate(&source, source.document_node(), &path, 8, 8, &mut cancelled).unwrap_err(),
            QualifiedPathFailure::Control(ControlFailure::Cancelled {
                domain: WorkDomain::XPathOperation
            })
        );
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = 2;
        assert!(matches!(
            evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                8,
                &mut InvocationControl::new(CancellationToken::new(), limits)
            ),
            Err(QualifiedPathFailure::Control(
                ControlFailure::BudgetExhausted {
                    domain: WorkDomain::XPathOperation,
                    ..
                }
            ))
        ));
        assert_eq!(
            evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                8,
                &mut InvocationControl::unbounded()
            )
            .unwrap()
            .len(),
            2
        );
    }

    #[test]
    fn qualified_parent_navigation_preserves_owner_and_charges_before_handoff() {
        let source = source();
        let path = parse(
            "/r/a/namespace::p",
            source.location(source.document_node()).clone(),
        )
        .unwrap();
        let nodes = evaluate(
            &source,
            source.document_node(),
            &path,
            8,
            8,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut parents = Vec::new();
        for node in nodes {
            let mut limits = WorkLimits::unbounded();
            limits.xpath_node_visits = 0;
            assert!(matches!(
                parent_controlled(
                    node,
                    &mut InvocationControl::new(CancellationToken::new(), limits)
                ),
                Err(ControlFailure::BudgetExhausted { .. })
            ));
            let mut cancelled =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 0);
            assert_eq!(
                parent_controlled(node, &mut cancelled).unwrap_err(),
                ControlFailure::Cancelled {
                    domain: WorkDomain::XPathNodeVisit
                }
            );
            let parent = parent_controlled(node, &mut InvocationControl::unbounded())
                .unwrap()
                .unwrap();
            assert!(parent.same_node(node.parent().unwrap()));
            let QualifiedSourceNode::Tree { document, id } = parent else {
                panic!("namespace parent must be an element");
            };
            assert!(std::ptr::eq(document, std::ptr::from_ref(&source)));
            assert_eq!(document.name(id).unwrap().local, "a");
            assert_eq!(document.location(id).resource, "memory:qualified-path.xml");
            parents.push(parent);
        }
        assert!(!parents[0].same_node(parents[1]));
    }
    #[test]
    fn partial_path_products_are_discarded_on_capacity_and_work_failure() {
        let source = source();
        let path = parse(
            "/r/a/namespace::*",
            source.location(source.document_node()).clone(),
        )
        .unwrap();
        assert_eq!(
            evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                3,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err(),
            QualifiedPathFailure::SequenceCapacity
        );
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = 1;
        assert!(matches!(
            evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                8,
                &mut InvocationControl::new(CancellationToken::new(), limits)
            ),
            Err(QualifiedPathFailure::Control(
                ControlFailure::BudgetExhausted { .. }
            ))
        ));
        let token = CancellationToken::new();
        token.cancel();
        assert!(matches!(
            evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                8,
                &mut InvocationControl::new(token, WorkLimits::unbounded())
            ),
            Err(QualifiedPathFailure::Control(
                ControlFailure::Cancelled { .. }
            ))
        ));
        assert_eq!(
            evaluate(
                &source,
                source.document_node(),
                &path,
                8,
                8,
                &mut InvocationControl::unbounded()
            )
            .unwrap()
            .len(),
            4
        );
    }

    #[test]
    fn owner_paths_preserve_origin_and_hidden_text_across_effective_documents() {
        let source = Document::from_parsed(
            parse_document(
                "memory:qualified-view.xml",
                b"<r xmlns:p='urn:p'> <a/> <a/> </r>",
                ParseLimits {
                    max_events: 20,
                    max_depth: 10,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let view = source
            .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap();
        let complete = source
            .derive_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap();
        let path = parse(
            "/r/a/namespace::p",
            source.location(source.document_node()).clone(),
        )
        .unwrap();
        let original = evaluate(
            &source,
            source.document_node(),
            &path,
            8,
            8,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        for effective in [&view, &complete] {
            let nodes = evaluate(
                effective,
                effective.document_node(),
                &path,
                8,
                8,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(nodes.len(), 2);
            for (first, second) in original.iter().zip(nodes) {
                assert!(first.same_node(second));
                assert_eq!(
                    first.namespace().unwrap().declaration(),
                    second.namespace().unwrap().declaration()
                );
            }
            let text = parse("/r/text()", source.location(source.document_node()).clone()).unwrap();
            assert!(
                evaluate(
                    effective,
                    effective.document_node(),
                    &text,
                    8,
                    8,
                    &mut InvocationControl::unbounded()
                )
                .unwrap()
                .is_empty()
            );
        }
        let text = parse("/r/text()", source.location(source.document_node()).clone()).unwrap();
        assert_eq!(
            evaluate(
                &source,
                source.document_node(),
                &text,
                8,
                8,
                &mut InvocationControl::unbounded()
            )
            .unwrap()
            .len(),
            3
        );
    }
}
