//! AR-0026 test-only occurrence reference; no runtime node representation is selected.

use std::collections::BTreeMap;

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};

use super::namespace_binding_reference::{BindingFailure, NamespaceOwner, XML_URI, resolve};

#[derive(Debug, Clone, Copy)]
struct NamespaceNode<'document> {
    document: &'document Document,
    origin: &'document Document,
    parent: NodeId,
    prefix: &'document str,
    uri: &'document str,
    declaration: &'document SourceLocation,
}

impl NamespaceNode<'_> {
    fn same_node(&self, other: &Self) -> bool {
        // Views created by the reference retain their prepared origin rather
        // than using effective-document addresses or namespace payload identity.
        NamespaceOwner::Source {
            origin: self.origin,
            element: self.parent,
        }
        .same_namespace(
            self.prefix,
            NamespaceOwner::Source {
                origin: other.origin,
                element: other.parent,
            },
            other.prefix,
        )
    }

    fn node_name_local(&self) -> Option<&str> {
        (!self.prefix.is_empty()).then_some(self.prefix)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ReferenceFailure {
    BindingCapacity,
    SequenceCapacity,
    CrossDocumentSequence,
    MixedEffectiveViews,
    InvisibleNode,
    Control(ControlFailure),
}

#[derive(Debug)]
enum EffectiveDocument<'document> {
    Prepared(&'document Document),
    Derived(Document),
}

#[derive(Debug)]
struct SourceReference<'document> {
    origin: &'document Document,
    effective: EffectiveDocument<'document>,
}

impl<'document> SourceReference<'document> {
    fn prepared(document: &'document Document) -> Self {
        Self {
            origin: document,
            effective: EffectiveDocument::Prepared(document),
        }
    }

    fn stripping(
        document: &'document Document,
        complete_oracle: bool,
        control: &mut InvocationControl,
    ) -> Result<Self, ControlFailure> {
        let effective = if complete_oracle {
            document.derive_stripping_all_element_whitespace(control)?
        } else {
            document.view_stripping_all_element_whitespace(control)?
        };
        // The origin association is established only by this owned derivation;
        // callers cannot pair arbitrary documents by URI, hash, or arena ID.
        Ok(Self {
            origin: document,
            effective: EffectiveDocument::Derived(effective),
        })
    }

    fn document(&self) -> &Document {
        match &self.effective {
            EffectiveDocument::Prepared(document) => document,
            EffectiveDocument::Derived(document) => document,
        }
    }

    fn namespaces(
        &self,
        parent: NodeId,
        control: &mut InvocationControl,
    ) -> Result<Vec<NamespaceNode<'_>>, ReferenceFailure> {
        occurrences_with_origin(self.document(), self.origin, parent, 16, control)
    }

    fn tree_node(
        &self,
        id: NodeId,
        control: &mut InvocationControl,
    ) -> Result<MixedNode<'_>, ReferenceFailure> {
        let document = self.document();
        let mut current = id;
        loop {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(ReferenceFailure::Control)?;
            let Some(parent) = document.parent(current) else {
                break;
            };
            let relationships = if document.kind(current) == NodeKind::Attribute {
                document.attributes(parent)
            } else {
                document.children(parent)
            };
            let mut visible = false;
            for candidate in relationships {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(ReferenceFailure::Control)?;
                if *candidate == current {
                    visible = true;
                    break;
                }
            }
            if !visible {
                return Err(ReferenceFailure::InvisibleNode);
            }
            current = parent;
        }
        Ok(MixedNode::Tree {
            document,
            origin: self.origin,
            id,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum MixedNode<'document> {
    Tree {
        document: &'document Document,
        origin: &'document Document,
        id: NodeId,
    },
    Namespace(NamespaceNode<'document>),
}

impl<'document> MixedNode<'document> {
    fn origin(self) -> &'document Document {
        match self {
            Self::Tree { origin, .. } => origin,
            Self::Namespace(node) => node.origin,
        }
    }

    fn document(self) -> &'document Document {
        match self {
            Self::Tree { document, .. } => document,
            Self::Namespace(node) => node.document,
        }
    }

    fn order_key(self) -> (usize, u8, &'document str) {
        match self {
            Self::Tree { document, id, .. } => (document.document_order(id), 0, ""),
            // Lexical prefix order is an experimental stable local choice,
            // not an archival processor convention or public guarantee.
            Self::Namespace(node) => (node.document.document_order(node.parent), 1, node.prefix),
        }
    }
}

fn normalize<'document>(
    nodes: &[MixedNode<'document>],
    max_nodes: usize,
    control: &mut InvocationControl,
) -> Result<Vec<MixedNode<'document>>, ReferenceFailure> {
    if nodes.len() > max_nodes {
        return Err(ReferenceFailure::SequenceCapacity);
    }
    let mut ordered: Vec<MixedNode<'document>> = Vec::new();
    for node in nodes {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(ReferenceFailure::Control)?;
        if let Some(first) = ordered.first()
            && !std::ptr::eq(first.origin(), node.origin())
        {
            return Err(ReferenceFailure::CrossDocumentSequence);
        }
        if let Some(first) = ordered.first()
            && !std::ptr::eq(first.document(), node.document())
        {
            return Err(ReferenceFailure::MixedEffectiveViews);
        }
        let mut insertion = ordered.len();
        let mut duplicate = false;
        // Complete charged insertion is intentionally simple, not an optimized
        // sorting candidate. Every comparison is cancellable and bounded.
        for (index, existing) in ordered.iter().enumerate() {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(ReferenceFailure::Control)?;
            match existing.order_key().cmp(&node.order_key()) {
                std::cmp::Ordering::Equal => {
                    duplicate = true;
                    break;
                }
                std::cmp::Ordering::Greater => {
                    insertion = index;
                    break;
                }
                std::cmp::Ordering::Less => {}
            }
        }
        if !duplicate {
            ordered.insert(insertion, *node);
        }
    }
    Ok(ordered)
}

fn occurrences<'document>(
    document: &'document Document,
    parent: NodeId,
    max_bindings: usize,
    control: &mut InvocationControl,
) -> Result<Vec<NamespaceNode<'document>>, ReferenceFailure> {
    occurrences_with_origin(document, document, parent, max_bindings, control)
}

fn occurrences_with_origin<'document>(
    document: &'document Document,
    origin: &'document Document,
    parent: NodeId,
    max_bindings: usize,
    control: &mut InvocationControl,
) -> Result<Vec<NamespaceNode<'document>>, ReferenceFailure> {
    charge(control)?;
    if document.kind(parent) != NodeKind::Element {
        return Ok(Vec::new());
    }
    // Borrowed payloads avoid copying inherited URI strings in this reference.
    // Tombstones count toward capacity: undeclarations must still hide ancestors.
    let ancestors =
        std::iter::successors(Some(parent), |element| document.parent(*element)).map(|element| {
            (
                document.namespace_declarations(element),
                document.location(element),
            )
        });
    let bindings = resolve(ancestors, document.location(parent), max_bindings, control).map_err(
        |failure| match failure {
            BindingFailure::Capacity => ReferenceFailure::BindingCapacity,
            BindingFailure::Control(failure) => ReferenceFailure::Control(failure),
        },
    )?;
    let mut result = Vec::new();
    for (prefix, (uri, declaration)) in bindings {
        if uri.is_empty() {
            continue;
        }
        charge(control)?;
        result.push(NamespaceNode {
            document,
            origin,
            parent,
            prefix,
            uri,
            declaration,
        });
    }
    Ok(result)
}

fn charge(control: &mut InvocationControl) -> Result<(), ReferenceFailure> {
    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(ReferenceFailure::Control)
}

#[cfg(test)]
mod tests {
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    use super::*;

    fn document(bytes: &[u8]) -> Document {
        Document::from_parsed(
            parse_document(
                "memory:namespace-reference.xml",
                bytes,
                ParseLimits {
                    max_events: 100,
                    max_depth: 20,
                },
            )
            .expect("well-formed fixture"),
        )
        .expect("source-derived XDM")
    }

    fn select(document: &Document, parent: NodeId) -> Vec<NamespaceNode<'_>> {
        occurrences(document, parent, 16, &mut InvocationControl::unbounded())
            .expect("bounded reference")
    }

    #[test]
    fn namespace_parent_handoff_reuses_effective_tree_path_evaluator() {
        use crate::xpath::path_experiment::{
            evaluate_location_path_controlled, parse_location_path,
        };

        let source = document(b"<r xmlns:p='urn:outer'> <a xmlns:p='urn:inner'> <b/> </a> </r>");
        let mut construction = InvocationControl::unbounded();
        let view = SourceReference::stripping(&source, false, &mut construction).unwrap();
        let complete = SourceReference::stripping(&source, true, &mut construction).unwrap();
        let owner_path =
            parse_location_path("/r/a", source.location(source.document_node()).clone()).unwrap();
        let children_path = parse_location_path("node()", owner_path.location.clone()).unwrap();
        let mut selected = Vec::new();
        for reference in [&view, &complete] {
            let mut control = InvocationControl::unbounded();
            let owners = evaluate_location_path_controlled(
                reference.document(),
                source.document_node(),
                &owner_path,
                &mut control,
            )
            .unwrap();
            assert_eq!(owners.len(), 1);
            let namespaces = reference.namespaces(owners[0], &mut control).unwrap();
            let namespace = namespaces
                .into_iter()
                .find(|node| node.prefix == "p")
                .unwrap();
            assert_eq!(namespace.uri, "urn:inner");
            assert_eq!(namespace.declaration, source.location(owners[0]));
            // This is a parent handoff, not namespace current-item execution:
            // only the namespace's actual parent can enter the ID-only evaluator.
            let children = evaluate_location_path_controlled(
                namespace.document,
                namespace.parent,
                &children_path,
                &mut control,
            )
            .unwrap();
            assert_eq!(children.len(), 1, "hidden whitespace must not reappear");
            assert_eq!(reference.document().kind(children[0]), NodeKind::Element);
            selected.push(namespace);
        }
        assert!(selected[0].same_node(&selected[1]));
        let original_owner = source.children(source.children(source.document_node())[0])[1];
        assert_eq!(
            source.children(original_owner).len(),
            3,
            "prepared input is unchanged"
        );
    }

    #[test]
    fn namespace_parent_handoff_keeps_the_same_cancellation_control() {
        use crate::xpath::path_experiment::{
            evaluate_location_path_controlled, parse_location_path,
        };

        let source = document(b"<r xmlns:p='urn:p'><a/></r>");
        let reference = SourceReference::prepared(&source);
        let token = CancellationToken::new();
        let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
        let root = source.children(source.document_node())[0];
        let namespace = reference.namespaces(root, &mut control).unwrap()[0];
        let path = parse_location_path("child::*", source.location(root).clone()).unwrap();
        token.cancel();
        assert!(matches!(
            evaluate_location_path_controlled(
                namespace.document,
                namespace.parent,
                &path,
                &mut control
            ),
            Err(ControlFailure::Cancelled { .. })
        ));
        assert_eq!(source.children(root).len(), 1);
    }

    #[test]
    fn inherited_bindings_shadow_and_default_undeclaration_removes_a_node() {
        let source = document(
            br#"<r xmlns="urn:default" xmlns:p="urn:outer"><a xmlns="" xmlns:p="urn:inner"><b/></a></r>"#,
        );
        let root = source.children(source.document_node())[0];
        let a = source.children(root)[0];
        let b = source.children(a)[0];
        let root_nodes = select(&source, root);
        assert_eq!(root_nodes.len(), 3);
        let default = root_nodes
            .iter()
            .find(|node| node.prefix.is_empty())
            .unwrap();
        assert_eq!(default.node_name_local(), None);
        assert_eq!(default.uri, "urn:default");
        let b_nodes = select(&source, b);
        assert_eq!(b_nodes.len(), 2);
        let p = b_nodes.iter().find(|node| node.prefix == "p").unwrap();
        assert_eq!(p.node_name_local(), Some("p"));
        assert_eq!(p.uri, "urn:inner");
        assert_eq!(p.parent, b);
        assert_eq!(p.declaration, source.location(a));
        assert_eq!(
            b_nodes
                .iter()
                .find(|node| node.prefix == "xml")
                .unwrap()
                .uri,
            XML_URI
        );
    }

    #[test]
    fn same_binding_on_different_elements_is_not_the_same_occurrence() {
        let source = document(b"<r xmlns:p='urn:p'><a/><b/></r>");
        let root = source.children(source.document_node())[0];
        let a = source.children(root)[0];
        let b = source.children(root)[1];
        let first = select(&source, a);
        let repeated = select(&source, a);
        let sibling = select(&source, b);
        for occurrence in &first {
            let repeat = repeated
                .iter()
                .find(|node| node.prefix == occurrence.prefix)
                .unwrap();
            let other = sibling
                .iter()
                .find(|node| node.prefix == occurrence.prefix)
                .unwrap();
            assert!(occurrence.same_node(repeat));
            assert!(!occurrence.same_node(other));
            assert_eq!(occurrence.uri, other.uri);
        }
        let separate = document(b"<r xmlns:p='urn:p'><a/><b/></r>");
        let separate_a = separate.children(separate.children(separate.document_node())[0])[0];
        assert_eq!(a, separate_a, "raw arena IDs collide across documents");
        for other in select(&separate, separate_a) {
            assert!(
                !first
                    .iter()
                    .find(|node| node.prefix == other.prefix)
                    .unwrap()
                    .same_node(&other)
            );
        }
    }

    #[test]
    fn occurrences_are_not_attributes_or_children_and_non_elements_select_nothing() {
        let source = document(b"<r xmlns:p='urn:p' a='v'>text</r>");
        let root = source.children(source.document_node())[0];
        let attribute = source.attributes(root)[0];
        let text = source.children(root)[0];
        assert_eq!(select(&source, root).len(), 2);
        assert_eq!(source.attributes(root).len(), 1);
        assert_eq!(source.children(root).len(), 1);
        for node in [source.document_node(), attribute, text] {
            assert!(select(&source, node).is_empty());
        }
    }

    #[test]
    fn capacity_and_work_failures_leave_prepared_source_unchanged() {
        let source = document(b"<r xmlns:p='urn:p' xmlns:q='urn:q'><a/></r>");
        let root = source.children(source.document_node())[0];
        let a = source.children(root)[0];
        for limit in [0, 1, 2] {
            assert_eq!(
                occurrences(&source, a, limit, &mut InvocationControl::unbounded()).unwrap_err(),
                ReferenceFailure::BindingCapacity
            );
        }
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = 4;
        let mut bounded = InvocationControl::new(CancellationToken::new(), limits);
        assert!(matches!(
            occurrences(&source, a, 16, &mut bounded),
            Err(ReferenceFailure::Control(ControlFailure::BudgetExhausted {
                domain: WorkDomain::XdmNode,
                ..
            }))
        ));
        assert_eq!(select(&source, a).len(), 3);
        assert_eq!(source.namespace_declarations(root).len(), 2);
        assert!(source.namespace_declarations(a).is_empty());
    }

    #[test]
    fn cancellation_during_ancestor_traversal_discards_partial_materialization() {
        let source = document(b"<r xmlns:p='urn:p'><a><b/></a></r>");
        let root = source.children(source.document_node())[0];
        let a = source.children(root)[0];
        let b = source.children(a)[0];
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 3);
        assert_eq!(
            occurrences(&source, b, 16, &mut control).unwrap_err(),
            ReferenceFailure::Control(ControlFailure::Cancelled {
                domain: WorkDomain::XdmNode
            })
        );
        assert_eq!(select(&source, b).len(), 2);
    }

    #[test]
    fn distinct_prefixes_binding_the_same_uri_are_distinct_nodes() {
        let source = document(b"<r xmlns:p='urn:same' xmlns:q='urn:same'/>");
        let root = source.children(source.document_node())[0];
        let nodes = select(&source, root);
        let p = nodes.iter().find(|node| node.prefix == "p").unwrap();
        let q = nodes.iter().find(|node| node.prefix == "q").unwrap();
        assert_eq!(p.uri, q.uri);
        assert!(!p.same_node(q));
    }

    #[test]
    fn concurrent_references_share_only_the_immutable_prepared_document() {
        let source = document(b"<r xmlns:p='urn:p'><a/></r>");
        let root = source.children(source.document_node())[0];
        let a = source.children(root)[0];
        let (first, second) = std::thread::scope(|scope| {
            let first = scope.spawn(|| select(&source, a));
            let second = scope.spawn(|| select(&source, a));
            (first.join().unwrap(), second.join().unwrap())
        });
        for node in &first {
            assert!(
                node.same_node(
                    second
                        .iter()
                        .find(|other| other.prefix == node.prefix)
                        .unwrap()
                )
            );
        }
        assert!(source.namespace_declarations(a).is_empty());
    }

    #[test]
    fn nearest_binding_reference_matches_the_complete_binding_lookup() {
        for fixture in [
            b"<r><a><b/></a></r>".as_slice(),
            b"<r xmlns='urn:d'><a xmlns=''><b/></a></r>",
            b"<r xmlns:p='urn:p'><a xmlns:p='urn:q'><b/></a></r>",
            b"<r xmlns:p='urn:p'><a xmlns:q='urn:q'><b xmlns:p='urn:new'/></a></r>",
        ] {
            let source = document(fixture);
            let root = source.children(source.document_node())[0];
            let a = source.children(root)[0];
            let b = source.children(a)[0];
            for element in [root, a, b] {
                let mut complete = source
                    .in_scope_namespaces(element)
                    .into_iter()
                    .filter(|binding| !binding.namespace.is_empty())
                    .map(|binding| (binding.prefix.unwrap_or_default(), binding.namespace))
                    .collect::<BTreeMap<_, _>>();
                complete.insert("xml".to_owned(), XML_URI.to_owned());
                let borrowed = select(&source, element)
                    .into_iter()
                    .map(|node| (node.prefix.to_owned(), node.uri.to_owned()))
                    .collect::<BTreeMap<_, _>>();
                assert_eq!(borrowed, complete);
            }
        }
    }

    #[test]
    fn namespace_identity_survives_shared_view_and_complete_derivation() {
        let source = document(b"<r xmlns:p='urn:p'> <a/> </r>");
        let root = source.children(source.document_node())[0];
        let whitespace = source.children(root)[0];
        let prepared = SourceReference::prepared(&source);
        let view = SourceReference::stripping(&source, false, &mut InvocationControl::unbounded())
            .unwrap();
        let complete =
            SourceReference::stripping(&source, true, &mut InvocationControl::unbounded()).unwrap();
        assert!(view.document().shares_node_storage_with(&source));
        assert!(!complete.document().shares_node_storage_with(&source));
        let original = prepared
            .namespaces(root, &mut InvocationControl::unbounded())
            .unwrap();
        for effective in [&view, &complete] {
            let nodes = effective
                .namespaces(root, &mut InvocationControl::unbounded())
                .unwrap();
            for node in &nodes {
                let before = original
                    .iter()
                    .find(|before| before.prefix == node.prefix)
                    .unwrap();
                assert!(node.same_node(before));
                assert_eq!(node.uri, before.uri);
                assert_eq!(node.declaration, before.declaration);
            }
            assert_eq!(effective.document().children(root).len(), 1);
            assert_eq!(
                effective
                    .tree_node(whitespace, &mut InvocationControl::unbounded())
                    .unwrap_err(),
                ReferenceFailure::InvisibleNode
            );
        }
        assert!(
            prepared
                .tree_node(whitespace, &mut InvocationControl::unbounded())
                .is_ok()
        );
        assert_eq!(source.children(root).len(), 3);
    }

    #[test]
    fn mixed_order_places_namespaces_before_attributes_and_preserves_content_order() {
        let source = document(b"<r xmlns:p='urn:p' a='v'>text<!--c--><?pi v?><b/></r>");
        let reference = SourceReference::prepared(&source);
        let root = source.children(source.document_node())[0];
        let mut control = InvocationControl::unbounded();
        let mut expected = vec![
            reference
                .tree_node(source.document_node(), &mut control)
                .unwrap(),
            reference.tree_node(root, &mut control).unwrap(),
        ];
        expected.extend(
            reference
                .namespaces(root, &mut control)
                .unwrap()
                .into_iter()
                .map(MixedNode::Namespace),
        );
        expected.push(
            reference
                .tree_node(source.attributes(root)[0], &mut control)
                .unwrap(),
        );
        for child in source.children(root) {
            expected.push(reference.tree_node(*child, &mut control).unwrap());
            if source.kind(*child) == NodeKind::Element {
                expected.extend(
                    reference
                        .namespaces(*child, &mut control)
                        .unwrap()
                        .into_iter()
                        .map(MixedNode::Namespace),
                );
            }
        }
        let mut reversed = expected.iter().rev().copied().collect::<Vec<_>>();
        reversed.extend(expected.iter().copied());
        let result = normalize(&reversed, 32, &mut control).unwrap();
        assert_eq!(result.len(), expected.len());
        assert_eq!(
            result
                .iter()
                .map(|node| node.order_key())
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|node| node.order_key())
                .collect::<Vec<_>>()
        );
        assert_eq!(source.children(root).len(), 4);
    }

    #[test]
    fn normalization_rejects_cross_document_identity_and_mixed_visibility() {
        let source = document(b"<r xmlns:p='urn:p'> <a/> </r>");
        let other = document(b"<r xmlns:p='urn:p'> <a/> </r>");
        let prepared = SourceReference::prepared(&source);
        let separate = SourceReference::prepared(&other);
        let view = SourceReference::stripping(&source, false, &mut InvocationControl::unbounded())
            .unwrap();
        let root = source.children(source.document_node())[0];
        let first = prepared
            .namespaces(root, &mut InvocationControl::unbounded())
            .unwrap()[0];
        let foreign = separate
            .namespaces(root, &mut InvocationControl::unbounded())
            .unwrap()[0];
        let visible = view
            .namespaces(root, &mut InvocationControl::unbounded())
            .unwrap()[0];
        assert!(!first.same_node(&foreign));
        assert!(first.same_node(&visible));
        assert_eq!(
            normalize(
                &[MixedNode::Namespace(first), MixedNode::Namespace(foreign)],
                2,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err(),
            ReferenceFailure::CrossDocumentSequence
        );
        assert_eq!(
            normalize(
                &[MixedNode::Namespace(first), MixedNode::Namespace(visible)],
                2,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err(),
            ReferenceFailure::MixedEffectiveViews
        );
    }

    #[test]
    fn mixed_normalization_is_bounded_and_cancellable_during_comparison() {
        let source = document(b"<r xmlns:p='urn:p' a='v'><a/></r>");
        let reference = SourceReference::prepared(&source);
        let root = source.children(source.document_node())[0];
        let mut nodes = reference
            .namespaces(root, &mut InvocationControl::unbounded())
            .unwrap()
            .into_iter()
            .map(MixedNode::Namespace)
            .collect::<Vec<_>>();
        nodes.push(
            reference
                .tree_node(
                    source.attributes(root)[0],
                    &mut InvocationControl::unbounded(),
                )
                .unwrap(),
        );
        assert_eq!(
            normalize(&nodes, 2, &mut InvocationControl::unbounded()).unwrap_err(),
            ReferenceFailure::SequenceCapacity
        );
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = 0;
        let mut bounded = InvocationControl::new(CancellationToken::new(), limits);
        assert!(matches!(
            normalize(&nodes, 3, &mut bounded),
            Err(ReferenceFailure::Control(ControlFailure::BudgetExhausted {
                domain: WorkDomain::XPathOperation,
                ..
            }))
        ));
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 1);
        assert_eq!(
            normalize(&nodes, 3, &mut cancelled).unwrap_err(),
            ReferenceFailure::Control(ControlFailure::Cancelled {
                domain: WorkDomain::XPathOperation
            })
        );
        assert_eq!(
            normalize(&nodes, 3, &mut InvocationControl::unbounded())
                .unwrap()
                .len(),
            3
        );
        assert_eq!(source.attributes(root).len(), 1);
    }

    fn wide_source(elements: usize, bindings: usize) -> Document {
        use std::fmt::Write as _;
        let mut xml = String::from("<r");
        for prefix in 0..bindings {
            write!(xml, " xmlns:p{prefix}='urn:binding:{prefix}'").unwrap();
        }
        xml.push('>');
        for _ in 0..elements {
            xml.push_str("<a/>");
        }
        xml.push_str("</r>");
        Document::from_parsed(
            parse_document(
                "memory:namespace-capacity.xml",
                xml.as_bytes(),
                ParseLimits {
                    max_events: elements * 2 + 10,
                    max_depth: 10,
                },
            )
            .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn eager_occurrence_index_amplifies_retention_without_changing_identity() {
        let source = wide_source(64, 8);
        let root = source.children(source.document_node())[0];
        let elements = std::iter::once(root)
            .chain(source.children(root).iter().copied())
            .collect::<Vec<_>>();
        let mut control = InvocationControl::unbounded();
        let eager = elements
            .iter()
            .map(|element| occurrences(&source, *element, 16, &mut control).unwrap())
            .collect::<Vec<_>>();
        let mut total = 0;
        for (element, retained) in elements.iter().zip(&eager) {
            let derived = occurrences(&source, *element, 16, &mut control).unwrap();
            assert_eq!(derived.len(), retained.len());
            assert!(
                derived
                    .iter()
                    .zip(retained)
                    .all(|(left, right)| left.same_node(right) && left.uri == right.uri)
            );
            total += derived.len();
        }
        assert_eq!(total, 65 * 9);
        let eager_payload = eager
            .iter()
            .map(|nodes| nodes.capacity() * std::mem::size_of::<NamespaceNode<'_>>())
            .sum::<usize>();
        let one = occurrences(&source, root, 16, &mut control).unwrap();
        assert!(eager_payload >= 65 * one.capacity() * std::mem::size_of::<NamespaceNode<'_>>());
        assert_eq!(source.namespace_declarations(root).len(), 8);
        assert!(
            source
                .children(root)
                .iter()
                .all(|child| source.namespace_declarations(*child).is_empty())
        );
    }

    #[test]
    #[ignore = "manual AR-0026 known-capacity probe; not an end-to-end benchmark"]
    fn measure_eager_vs_derived_namespace_occurrence_capacity() {
        for (elements, bindings) in [(8, 4), (64, 8), (512, 16)] {
            let source = wide_source(elements, bindings);
            let root = source.children(source.document_node())[0];
            let owners = std::iter::once(root)
                .chain(source.children(root).iter().copied())
                .collect::<Vec<_>>();
            let mut control = InvocationControl::unbounded();
            let eager = owners
                .iter()
                .map(|owner| occurrences(&source, *owner, bindings + 1, &mut control).unwrap())
                .collect::<Vec<_>>();
            let eager_bytes = eager.capacity() * std::mem::size_of::<Vec<NamespaceNode<'_>>>()
                + eager
                    .iter()
                    .map(|nodes| nodes.capacity() * std::mem::size_of::<NamespaceNode<'_>>())
                    .sum::<usize>();
            let mut derived_high_water = 0;
            let mut selected = 0;
            for owner in owners {
                let nodes = occurrences(&source, owner, bindings + 1, &mut control).unwrap();
                selected += nodes.len();
                derived_high_water = derived_high_water
                    .max(nodes.capacity() * std::mem::size_of::<NamespaceNode<'_>>());
            }
            println!(
                "namespace_capacity elements={} bindings={} occurrences={} occurrence_size={} eager_known_vec_bytes={} derived_one_selection_known_vec_bytes={}",
                elements + 1,
                bindings + 1,
                selected,
                std::mem::size_of::<NamespaceNode<'_>>(),
                eager_bytes,
                derived_high_water
            );
        }
    }
}
