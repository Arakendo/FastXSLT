//! AR-0026 temporary-owner reference, not an alternate executor or admitted axis.

#[path = "namespace_copy_reference_tests.rs"]
mod namespace_copy_reference_tests;

use super::super::result_tree::ResultNode;
use super::{TemporaryNodeKind, TemporaryTree, materialize_result_nodes};
use crate::execution_control_experiment::{
    CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
};
use crate::xdm::namespace_binding_reference::{
    BindingFailure, InvocationScope, NamespaceOwner, XML_URI, resolve,
};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{
    ExpandedName, NamespaceBinding, ParseLimits, parse_document,
};
use std::collections::BTreeMap;

#[derive(Debug)]
struct Occurrence<'a> {
    invocation: &'a InvocationScope,
    tree: &'a TemporaryTree,
    parent: usize,
    prefix: &'a str,
    uri: &'a str,
}

impl Occurrence<'_> {
    fn same_node(&self, other: &Self) -> bool {
        NamespaceOwner::Temporary {
            invocation: self.invocation,
            tree: self.tree.identity,
            element: self.parent,
        }
        .same_namespace(
            self.prefix,
            NamespaceOwner::Temporary {
                invocation: other.invocation,
                tree: other.tree.identity,
                element: other.parent,
            },
            other.prefix,
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Failure {
    Capacity,
    Control(ControlFailure),
}

fn select<'a>(
    invocation: &'a InvocationScope,
    tree: &'a TemporaryTree,
    parent: usize,
    max_bindings: usize,
    control: &mut InvocationControl,
) -> Result<Vec<Occurrence<'a>>, Failure> {
    let charge = |control: &mut InvocationControl| {
        control
            .charge(WorkDomain::XdmNode, 1)
            .map_err(Failure::Control)
    };
    charge(control)?;
    if !matches!(tree.nodes[parent].kind, TemporaryNodeKind::Element { .. }) {
        return Ok(Vec::new());
    }
    let ancestors =
        std::iter::successors(Some(parent), |node| tree.nodes[*node].parent).map(|node| {
            let declarations = match &tree.nodes[node].kind {
                TemporaryNodeKind::Element { namespaces, .. } => namespaces.as_slice(),
                _ => &[],
            };
            (declarations, ())
        });
    let bindings =
        resolve(ancestors, (), max_bindings, control).map_err(|failure| match failure {
            BindingFailure::Capacity => Failure::Capacity,
            BindingFailure::Control(failure) => Failure::Control(failure),
        })?;
    let mut result = Vec::new();
    for (prefix, (uri, ())) in bindings {
        if uri.is_empty() {
            continue;
        }
        charge(control)?;
        result.push(Occurrence {
            invocation,
            tree,
            parent,
            prefix,
            uri,
        });
    }
    Ok(result)
}

fn element(local: &str, bindings: &[(&str, &str)], children: Vec<ResultNode>) -> ResultNode {
    ResultNode::Element {
        name: ExpandedName {
            namespace: None,
            local: local.to_owned(),
        },
        namespaces: bindings
            .iter()
            .map(|(prefix, uri)| NamespaceBinding {
                prefix: (!prefix.is_empty()).then(|| (*prefix).to_owned()),
                namespace: (*uri).to_owned(),
            })
            .collect::<Vec<_>>()
            .into(),
        attributes: Vec::new(),
        children,
    }
}

fn fixture() -> Vec<ResultNode> {
    vec![element(
        "r",
        &[("", "urn:d"), ("p", "urn:outer")],
        vec![
            element(
                "a",
                &[("", ""), ("p", "urn:inner")],
                vec![element("b", &[], vec![])],
            ),
            element("c", &[], vec![]),
        ],
    )]
}

fn values(nodes: &[Occurrence<'_>]) -> BTreeMap<String, String> {
    nodes
        .iter()
        .map(|node| (node.prefix.to_owned(), node.uri.to_owned()))
        .collect()
}

#[test]
fn real_temporary_materialization_matches_source_binding_navigation() {
    let source = Document::from_parsed(parse_document("memory:reference.xml",
        b"<r xmlns='urn:d' xmlns:p='urn:outer'><a xmlns='' xmlns:p='urn:inner'><b/></a><c/></r>",
        ParseLimits { max_events: 100, max_depth: 10 }).unwrap()).unwrap();
    let root = source.children(source.document_node())[0];
    let a = source.children(root)[0];
    let b = source.children(a)[0];
    let c = source.children(root)[1];
    let mut control = InvocationControl::unbounded();
    let tree = materialize_result_nodes(&fixture(), "reference", &mut control).unwrap();
    let scope = InvocationScope::default();
    for (source_node, temporary_node) in [root, a, b, c].into_iter().zip(0..4) {
        let mut expected = source
            .in_scope_namespaces(source_node)
            .into_iter()
            .filter(|binding| !binding.namespace.is_empty())
            .map(|binding| (binding.prefix.unwrap_or_default(), binding.namespace))
            .collect::<BTreeMap<_, _>>();
        expected.insert("xml".to_owned(), XML_URI.to_owned());
        let actual = select(&scope, &tree, temporary_node, 16, &mut control).unwrap();
        assert_eq!(values(&actual), expected);
        assert!(actual.iter().all(|node| node.parent == temporary_node));
        assert_eq!(
            tree.nodes[temporary_node].children.len(),
            source.children(source_node).len()
        );
    }
}

#[test]
fn cloned_frame_preserves_identity_but_new_tree_and_new_invocation_do_not() {
    let scope = InvocationScope::default();
    let other_scope = InvocationScope::default();
    let mut control = InvocationControl::unbounded();
    let tree = materialize_result_nodes(&fixture(), "reference", &mut control).unwrap();
    let cloned = tree.clone();
    let fresh = materialize_result_nodes(&fixture(), "reference", &mut control).unwrap();
    let separate =
        materialize_result_nodes(&fixture(), "reference", &mut InvocationControl::unbounded())
            .unwrap();
    assert_eq!(
        tree.identity, separate.identity,
        "IDs restart in a different invocation"
    );
    let original = select(&scope, &tree, 0, 16, &mut control).unwrap();
    for (owner, candidate, equal) in [
        (&scope, &cloned, true),
        (&scope, &fresh, false),
        (&other_scope, &separate, false),
    ] {
        let nodes = select(owner, candidate, 0, 16, &mut control).unwrap();
        for (before, after) in original.iter().zip(&nodes) {
            assert_eq!(before.same_node(after), equal);
            assert_eq!(before.uri, after.uri);
        }
    }
    let descendant = select(&scope, &tree, 3, 16, &mut control).unwrap();
    assert!(!original[0].same_node(&descendant[0]));
}

#[test]
fn non_elements_have_no_namespace_axis_and_selection_has_local_focus() {
    let nodes = vec![element(
        "r",
        &[("p", "urn:p")],
        vec![ResultNode::Text("text".to_owned())],
    )];
    let mut control = InvocationControl::unbounded();
    let tree = materialize_result_nodes(&nodes, "reference", &mut control).unwrap();
    let scope = InvocationScope::default();
    let selected = select(&scope, &tree, 0, 16, &mut control).unwrap();
    let focus = selected
        .iter()
        .enumerate()
        .map(|(index, node)| (node.prefix, index + 1, selected.len()))
        .collect::<Vec<_>>();
    assert_eq!(focus, vec![("p", 1, 2), ("xml", 2, 2)]);
    assert_eq!(tree.nodes[0].children, vec![1]);
    assert!(
        select(&scope, &tree, 1, 16, &mut control)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn temporary_occurrence_failures_discard_partial_products_without_tree_mutation() {
    let mut control = InvocationControl::unbounded();
    let tree = materialize_result_nodes(&fixture(), "reference", &mut control).unwrap();
    let before = tree.clone();
    let scope = InvocationScope::default();
    assert_eq!(
        select(&scope, &tree, 2, 1, &mut control).unwrap_err(),
        Failure::Capacity
    );
    let mut limits = WorkLimits::unbounded();
    limits.xdm_nodes = 3;
    let mut bounded = InvocationControl::new(CancellationToken::new(), limits);
    assert!(matches!(
        select(&scope, &tree, 2, 16, &mut bounded),
        Err(Failure::Control(ControlFailure::BudgetExhausted {
            domain: WorkDomain::XdmNode,
            ..
        }))
    ));
    let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 4);
    assert_eq!(
        select(&scope, &tree, 2, 16, &mut cancelled).unwrap_err(),
        Failure::Control(ControlFailure::Cancelled {
            domain: WorkDomain::XdmNode
        })
    );
    assert_eq!(tree, before);
    assert_eq!(select(&scope, &tree, 2, 16, &mut control).unwrap().len(), 2);
}

#[test]
fn selected_namespace_sizes_reuse_charged_runtime_focus_comparisons() {
    use super::super::{
        FocusEqualityOperand, MultipleMatchPolicy, SequenceFocus, compile_resource,
        evaluate_context_focus_equality,
    };
    use super::{RuntimeGlobals, SequenceInputs};
    use crate::resources::{ResourceLimits, ResourceSetBuilder};
    use crate::xdm::owned_tree_experiment::SourceLocation;
    use std::cell::RefCell;

    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 1024, 1024));
    resources.admit("memory:focus.xsl", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, "memory:focus.xsl").unwrap();
    let globals = RuntimeGlobals::default();
    let dynamic_documents = RefCell::default();
    let inputs = SequenceInputs {
        program: &program,
        source: None,
        request_id: "namespace-focus",
        globals: &globals,
        multiple_match_policy: MultipleMatchPolicy::UseLast,
        document_rooted_matches: RefCell::default(),
        key_indexes: RefCell::default(),
        complete_atomic_frame_clones: false,
        resource_snapshot: None,
        denied_resources: None,
        dynamic_documents: &dynamic_documents,
    };
    // A supplied call-site fixture location, not inferred declaration provenance
    // on the temporary namespace occurrence or a compiler offset-parity claim.
    let location = SourceLocation {
        resource: "memory:focus.xsl".to_owned(),
        span: 0..1,
    };
    let scope = InvocationScope::default();
    let mut selection_control = InvocationControl::unbounded();
    let tree = materialize_result_nodes(&fixture(), "reference", &mut selection_control).unwrap();
    let original = tree.clone();
    for owner in [0, 1, 2, 3] {
        let selected = select(&scope, &tree, owner, 16, &mut selection_control).unwrap();
        let mut control = InvocationControl::unbounded();
        for (index, occurrence) in selected.iter().enumerate() {
            assert_eq!(occurrence.parent, owner);
            let focus = Some(SequenceFocus {
                position: index + 1,
                size: selected.len(),
            });
            let last = evaluate_context_focus_equality(
                &inputs,
                focus,
                FocusEqualityOperand::Position,
                FocusEqualityOperand::Size,
                &location,
                &mut control,
            )
            .unwrap();
            let first = evaluate_context_focus_equality(
                &inputs,
                focus,
                FocusEqualityOperand::Position,
                FocusEqualityOperand::Static(1),
                &location,
                &mut control,
            )
            .unwrap();
            assert_eq!(last, index + 1 == selected.len());
            assert_eq!(first, index == 0);
        }
        assert_eq!(
            control.consumed(WorkDomain::XPathOperation),
            selected.len() * 2
        );
    }
    assert_focus_control_failures(&inputs, &location);
    assert_eq!(tree, original);
}

fn assert_focus_control_failures(
    inputs: &super::SequenceInputs<'_>,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
) {
    use super::super::{FocusEqualityOperand, SequenceFocus, evaluate_context_focus_equality};
    let focus = Some(SequenceFocus {
        position: 1,
        size: 3,
    });
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 1);
    // The first successful charge signals cancellation; the next real charge
    // observes it, matching the existing cooperative control contract.
    assert!(
        !evaluate_context_focus_equality(
            inputs,
            focus,
            FocusEqualityOperand::Position,
            FocusEqualityOperand::Size,
            location,
            &mut cancelled,
        )
        .unwrap()
    );
    let failure = evaluate_context_focus_equality(
        inputs,
        focus,
        FocusEqualityOperand::Position,
        FocusEqualityOperand::Size,
        location,
        &mut cancelled,
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXCT0001");
    assert_eq!(failure.request_id.as_deref(), Some("namespace-focus"));
    let absent = evaluate_context_focus_equality(
        inputs,
        None,
        FocusEqualityOperand::Position,
        FocusEqualityOperand::Size,
        location,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(absent.code, "XPDY0002");
    assert_eq!(absent.location.as_ref(), Some(location));
}
