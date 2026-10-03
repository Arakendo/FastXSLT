//! ADR-0021 private source-node carrier and bounded namespace selection.
//! Temporary owners and general compiled axes are not admitted by this slice.

use std::collections::BTreeMap;

use super::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

const XML_URI: &str = "http://www.w3.org/XML/1998/namespace";

#[derive(Debug, Clone, Copy)]
pub(crate) struct NamespaceOccurrence<'a> {
    document: &'a Document,
    parent: NodeId,
    prefix: &'a str,
    uri: &'a str,
    declaration: &'a SourceLocation,
}

// The count AVT is the first caller; focus migration will consume these methods.
#[allow(dead_code)]
impl<'a> NamespaceOccurrence<'a> {
    pub(crate) fn prefix(self) -> &'a str {
        self.prefix
    }
    pub(crate) fn string_value(self) -> &'a str {
        self.uri
    }
    pub(crate) fn node_name_local(self) -> Option<&'a str> {
        (!self.prefix.is_empty()).then_some(self.prefix)
    }
    pub(crate) fn declaration(self) -> &'a SourceLocation {
        self.declaration
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum QualifiedSourceNode<'a> {
    Tree { document: &'a Document, id: NodeId },
    Namespace(NamespaceOccurrence<'a>),
}

#[allow(dead_code)]
impl<'a> QualifiedSourceNode<'a> {
    /// Same-origin order only. No cross-document ranking is selected here.
    pub(crate) fn compare_same_origin(self, other: Self) -> Option<std::cmp::Ordering> {
        fn key(node: QualifiedSourceNode<'_>) -> (&Document, (usize, u8, usize, &str)) {
            match node {
                QualifiedSourceNode::Namespace(node) => (
                    node.document,
                    (node.document.document_order(node.parent), 1, 0, node.prefix),
                ),
                QualifiedSourceNode::Tree { document, id } => {
                    let order = document.document_order(id);
                    if document.kind(id) == NodeKind::Attribute {
                        let parent = document.parent(id).expect("source attribute has an owner");
                        (document, (document.document_order(parent), 2, order, ""))
                    } else {
                        (document, (order, 0, 0, ""))
                    }
                }
            }
        }
        let (left, left_key) = key(self);
        let (right, right_key) = key(other);
        left.same_origin(right).then(|| left_key.cmp(&right_key))
    }

    pub(crate) fn same_node(self, other: Self) -> bool {
        match (self, other) {
            (
                Self::Tree { document, id },
                Self::Tree {
                    document: other,
                    id: other_id,
                },
            ) => document.same_origin(other) && id == other_id,
            (Self::Namespace(node), Self::Namespace(other)) => {
                node.document.same_origin(other.document)
                    && node.parent == other.parent
                    && node.prefix == other.prefix
            }
            _ => false,
        }
    }

    pub(crate) fn parent(self) -> Option<Self> {
        match self {
            Self::Tree { document, id } => {
                document.parent(id).map(|id| Self::Tree { document, id })
            }
            Self::Namespace(node) => Some(Self::Tree {
                document: node.document,
                id: node.parent,
            }),
        }
    }

    pub(crate) fn namespace(self) -> Option<NamespaceOccurrence<'a>> {
        match self {
            Self::Namespace(node) => Some(node),
            Self::Tree { .. } => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NamespaceFailure {
    BindingCapacity,
    Control(ControlFailure),
}

fn charge(control: &mut InvocationControl) -> Result<(), NamespaceFailure> {
    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(NamespaceFailure::Control)
}

impl Document {
    pub(crate) fn namespace_nodes_controlled(
        &self,
        parent: NodeId,
        max_bindings: usize,
        control: &mut InvocationControl,
    ) -> Result<Vec<QualifiedSourceNode<'_>>, NamespaceFailure> {
        charge(control)?;
        if self.kind(parent) != NodeKind::Element {
            return Ok(Vec::new());
        }
        if max_bindings == 0 {
            return Err(NamespaceFailure::BindingCapacity);
        }
        // Charge implicit binding retention before allocating its map entry.
        charge(control)?;
        let mut bindings = BTreeMap::from([("xml", (XML_URI, self.location(parent)))]);
        for ancestor in std::iter::successors(Some(parent), |id| self.parent(*id)) {
            charge(control)?;
            for binding in self.namespace_declarations(ancestor) {
                charge(control)?;
                let prefix = binding.prefix.as_deref().unwrap_or("");
                if bindings.contains_key(prefix) {
                    continue;
                }
                if bindings.len() == max_bindings {
                    return Err(NamespaceFailure::BindingCapacity);
                }
                // The declaration visit is charged before this bounded retention.
                bindings.insert(
                    prefix,
                    (binding.namespace.as_str(), self.location(ancestor)),
                );
            }
        }
        let mut result = Vec::new();
        // BTreeMap uses str::cmp: empty first, case-sensitive, no normalization.
        for (prefix, (uri, declaration)) in bindings {
            if uri.is_empty() {
                continue;
            } // An undeclaration blocks inheritance.
            charge(control)?;
            result.push(QualifiedSourceNode::Namespace(NamespaceOccurrence {
                document: self,
                parent,
                prefix,
                uri,
                declaration,
            }));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn source(bytes: &[u8]) -> Document {
        Document::from_parsed(
            parse_document(
                "memory:qualified.xml",
                bytes,
                ParseLimits {
                    max_events: 100,
                    max_depth: 20,
                },
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn selected(source: &Document, parent: NodeId) -> Vec<QualifiedSourceNode<'_>> {
        source
            .namespace_nodes_controlled(parent, 32, &mut InvocationControl::unbounded())
            .unwrap()
    }

    #[test]
    fn prefix_order_is_empty_then_case_sensitive_utf8_with_xml_unprivileged() {
        let source = source(
            "<r xmlns='urn:d' xmlns:z='urn:z' xmlns:A='urn:A' xmlns:a='urn:a' xmlns:é='urn:e'/>"
                .as_bytes(),
        );
        let root = source.children(source.document_node())[0];
        let nodes = selected(&source, root);
        assert_eq!(
            nodes
                .iter()
                .map(|node| node.namespace().unwrap().prefix())
                .collect::<Vec<_>>(),
            ["", "A", "a", "xml", "z", "é"]
        );
        assert_eq!(nodes[0].namespace().unwrap().node_name_local(), None);
        assert_eq!(nodes[3].namespace().unwrap().string_value(), XML_URI);
        for node in nodes {
            assert!(node.parent().unwrap().same_node(QualifiedSourceNode::Tree {
                document: &source,
                id: root
            }));
        }
    }

    #[test]
    fn origin_qualification_survives_views_and_complete_derivation_not_reparsing() {
        let bytes = b"<r xmlns:p='urn:p'> <a/> </r>";
        let source = source(bytes);
        let root = source.children(source.document_node())[0];
        let a = source.children(root)[1];
        let view = source
            .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap();
        let complete = source
            .derive_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap();
        let other = super::tests::source(bytes);
        let original = selected(&source, a);
        for effective in [&view, &complete] {
            let derived = selected(effective, a);
            for (first, second) in original.iter().zip(derived) {
                assert!(first.same_node(second));
                assert_eq!(
                    first.namespace().unwrap().declaration(),
                    second.namespace().unwrap().declaration()
                );
            }
            assert_eq!(effective.children(root), &[a]);
        }
        assert!(!original[0].same_node(selected(&other, a)[0]));
        assert!(!original[0].same_node(selected(&source, root)[0]));
        assert_eq!(source.children(root).len(), 3);
    }

    #[test]
    fn bounded_selection_preserves_shadowing_and_default_undeclaration() {
        let source =
            source(b"<r xmlns='urn:d' xmlns:p='urn:outer'><a xmlns='' xmlns:p='urn:inner'/></r>");
        let root = source.children(source.document_node())[0];
        let a = source.children(root)[0];
        let nodes = selected(&source, a);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].namespace().unwrap().prefix(), "p");
        assert_eq!(nodes[0].namespace().unwrap().string_value(), "urn:inner");
        assert_eq!(
            source
                .namespace_nodes_controlled(a, 2, &mut InvocationControl::unbounded())
                .unwrap_err(),
            NamespaceFailure::BindingCapacity
        );
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = 1;
        assert!(matches!(
            source.namespace_nodes_controlled(
                a,
                32,
                &mut InvocationControl::new(CancellationToken::new(), limits)
            ),
            Err(NamespaceFailure::Control(
                ControlFailure::BudgetExhausted { .. }
            ))
        ));
        let token = CancellationToken::new();
        token.cancel();
        assert!(matches!(
            source.namespace_nodes_controlled(
                a,
                32,
                &mut InvocationControl::new(token, WorkLimits::unbounded())
            ),
            Err(NamespaceFailure::Control(ControlFailure::Cancelled { .. }))
        ));
        assert_eq!(selected(&source, a).len(), 2);
        assert!(selected(&source, source.document_node()).is_empty());
    }

    #[test]
    fn selector_matches_complete_binding_lookup_without_cloning_payloads() {
        let source = source(b"<r xmlns='urn:d' xmlns:p='urn:outer'><a xmlns='' xmlns:p='urn:inner' xmlns:q='urn:q'><b/></a></r>");
        let root = source.children(source.document_node())[0];
        let a = source.children(root)[0];
        let b = source.children(a)[0];
        for parent in [root, a, b] {
            // Existing complete root-first lookup is independent of the selector's
            // nearest-first tombstones. It owns copied strings as the oracle.
            let mut expected = source
                .in_scope_namespaces(parent)
                .into_iter()
                .filter(|binding| !binding.namespace.is_empty())
                .map(|binding| (binding.prefix.unwrap_or_default(), binding.namespace))
                .collect::<BTreeMap<_, _>>();
            expected
                .entry("xml".into())
                .or_insert_with(|| XML_URI.into());
            let actual = selected(&source, parent)
                .into_iter()
                .map(|node| {
                    let occurrence = node.namespace().unwrap();
                    (
                        occurrence.prefix().to_owned(),
                        occurrence.string_value().to_owned(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn selection_charge_boundary_includes_implicit_and_output_retention() {
        let source = source(b"<r xmlns:p='urn:p'/>");
        let root = source.children(source.document_node())[0];
        // One selection, one implicit retention, two ancestor visits,
        // one authored declaration, and two retained occurrences = seven.
        for allowance in 0..=7 {
            let mut limits = WorkLimits::unbounded();
            limits.xdm_nodes = allowance;
            let result = source.namespace_nodes_controlled(
                root,
                2,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowance < 7 {
                assert!(matches!(
                    result,
                    Err(NamespaceFailure::Control(ControlFailure::BudgetExhausted {
                        domain: WorkDomain::XdmNode,
                        ..
                    }))
                ));
            } else {
                assert_eq!(result.unwrap().len(), 2);
            }
        }
        assert_eq!(selected(&source, root).len(), 2);
    }
}
