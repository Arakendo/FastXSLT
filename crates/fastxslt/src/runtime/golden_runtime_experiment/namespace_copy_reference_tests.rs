//! AR-0026 test-only modern attachment subset over existing semantic results.
//! Inputs are already-valid namespace occurrences and already-constructed children.

use super::super::super::{
    ExecutionFailure, FailureCategory, ResultNode, SemanticResult, control_failure, failure,
    serialize_xml, serialize_xml_complete_namespace_reference,
};
use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};

enum Item<'a> {
    Namespace { prefix: &'a str, uri: &'a str },
    Child(ResultNode),
}

fn invalid(code: &'static str) -> ExecutionFailure {
    failure(
        code,
        FailureCategory::Invalid,
        Some("namespace-copy"),
        "namespace attachment reference rejected content",
    )
}

fn capacity_failure() -> ExecutionFailure {
    failure(
        "FXCT0002",
        FailureCategory::Limit,
        Some("namespace-copy"),
        "namespace attachment reference capacity exceeded",
    )
}

fn assemble(
    destination: Option<ExpandedName>,
    items: Vec<Item<'_>>,
    max_items: usize,
    max_namespaces: usize,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    if items.len() > max_items {
        return Err(capacity_failure());
    }
    let mut namespaces: Vec<NamespaceBinding> = Vec::new();
    let mut children = Vec::new();
    if destination.is_some() {
        control
            .charge(WorkDomain::ResultNode, 1)
            .map_err(|error| control_failure(error, "namespace-copy"))?;
    }
    for item in items {
        control
            .charge(WorkDomain::ResultNode, 1)
            .map_err(|error| control_failure(error, "namespace-copy"))?;
        match item {
            Item::Namespace { prefix, uri } => {
                let Some(name) = &destination else {
                    return Err(invalid("XTDE0420"));
                };
                if !children.is_empty() {
                    return Err(invalid("XTDE0410"));
                }
                if prefix.is_empty() && name.namespace.is_none() {
                    return Err(invalid("XTDE0440"));
                }
                let bytes = prefix
                    .len()
                    .checked_add(uri.len())
                    .ok_or_else(capacity_failure)?;
                control
                    .charge(WorkDomain::ResultTextByte, bytes)
                    .map_err(|error| control_failure(error, "namespace-copy"))?;
                let mut duplicate = false;
                for existing in &namespaces {
                    control
                        .charge(WorkDomain::ResultNode, 1)
                        .map_err(|error| control_failure(error, "namespace-copy"))?;
                    if existing.prefix.as_deref().unwrap_or("") == prefix {
                        if existing.namespace != uri {
                            return Err(invalid("XTDE0430"));
                        }
                        duplicate = true;
                        break;
                    }
                }
                if !duplicate {
                    if namespaces.len() == max_namespaces {
                        return Err(capacity_failure());
                    }
                    namespaces.push(NamespaceBinding {
                        prefix: (!prefix.is_empty()).then(|| prefix.to_owned()),
                        namespace: uri.to_owned(),
                    });
                }
            }
            Item::Child(ResultNode::Text(value)) if value.is_empty() => {}
            // Attributes, atomization, document flattening, text merging, and
            // general namespace construction are not implemented by this probe.
            Item::Child(
                child @ (ResultNode::PendingAttribute(_)
                | ResultNode::Xslt10RecoverableAttribute(_)),
            ) => {
                drop(child);
                return Err(failure(
                    "FXNSREF0001",
                    FailureCategory::Unsupported,
                    Some("namespace-copy"),
                    "reference does not accept attribute items",
                ));
            }
            Item::Child(child) => children.push(child),
        }
    }
    Ok(SemanticResult {
        children: match destination {
            Some(name) => vec![ResultNode::Element {
                name,
                namespaces: namespaces.into(),
                attributes: Vec::new(),
                children,
            }],
            None => children,
        },
    })
}

fn name(namespace: Option<&str>) -> ExpandedName {
    ExpandedName {
        namespace: namespace.map(str::to_owned),
        local: "out".to_owned(),
    }
}

fn namespace<'a>(prefix: &'a str, uri: &'a str) -> Item<'a> {
    Item::Namespace { prefix, uri }
}

fn bytes(result: &SemanticResult) -> String {
    use super::super::super::compile_resource;
    use crate::resources::{ResourceLimits, ResourceSetBuilder};
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(1, 1024, 1024));
    builder.admit("memory:copy.xsl", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    let program = compile_resource(&builder.seal(), "memory:copy.xsl").unwrap();
    let normal = serialize_xml(
        result,
        &program.output,
        "namespace-copy",
        4096,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let complete = serialize_xml_complete_namespace_reference(
        result,
        &program.output,
        "namespace-copy",
        4096,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(normal, complete);
    normal
}

#[test]
fn temporary_namespace_copy_owns_payload_after_tree_and_occurrences_are_dropped() {
    let result = {
        let mut control = InvocationControl::unbounded();
        let tree =
            super::materialize_result_nodes(&super::fixture(), "reference", &mut control).unwrap();
        let scope = super::InvocationScope::default();
        let selected = super::select(&scope, &tree, 2, 16, &mut control).unwrap();
        assemble(
            Some(name(None)),
            selected
                .iter()
                .map(|node| namespace(node.prefix, node.uri))
                .collect(),
            16,
            16,
            &mut control,
        )
        .unwrap()
    };
    assert_eq!(
        bytes(&result),
        "<out xmlns:p=\"urn:inner\" xmlns:xml=\"http://www.w3.org/XML/1998/namespace\"></out>"
    );
    let ResultNode::Element {
        attributes,
        children,
        ..
    } = &result.children[0]
    else {
        panic!("element result");
    };
    assert!(attributes.is_empty());
    assert!(children.is_empty());
}

#[test]
fn source_namespace_copy_uses_the_same_attachment_and_owned_result() {
    use crate::xdm::namespace_binding_reference::resolve;
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
    let result = {
        let document = Document::from_parsed(
            parse_document(
                "memory:copy-source.xml",
                b"<r xmlns:p='urn:inner'><b/></r>",
                ParseLimits {
                    max_events: 20,
                    max_depth: 5,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let root = document.children(document.document_node())[0];
        let b = document.children(root)[0];
        let mut control = InvocationControl::unbounded();
        let rows = std::iter::successors(Some(b), |node| document.parent(*node)).map(|node| {
            (
                document.namespace_declarations(node),
                document.location(node),
            )
        });
        let selected = resolve(rows, document.location(b), 16, &mut control).unwrap();
        let items = selected
            .iter()
            .filter(|(_, (uri, _))| !uri.is_empty())
            .map(|(prefix, (uri, _))| namespace(prefix, uri))
            .collect();
        assemble(Some(name(None)), items, 16, 16, &mut control).unwrap()
    };
    assert_eq!(
        bytes(&result),
        "<out xmlns:p=\"urn:inner\" xmlns:xml=\"http://www.w3.org/XML/1998/namespace\"></out>"
    );
}

#[test]
fn duplicate_binding_is_collapsed_and_different_uri_conflict_is_not_last_wins() {
    let result = assemble(
        Some(name(None)),
        vec![namespace("p", "urn:p"), namespace("p", "urn:p")],
        2,
        1,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(bytes(&result), "<out xmlns:p=\"urn:p\"></out>");
    let error = assemble(
        Some(name(None)),
        vec![namespace("p", "urn:p"), namespace("p", "urn:q")],
        2,
        2,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(error.code, "XTDE0430");
    assert_eq!(error.request_id.as_deref(), Some("namespace-copy"));
}

#[test]
fn namespace_after_child_and_document_attachment_are_explicit_errors() {
    for child in [
        ResultNode::Text("text".to_owned()),
        ResultNode::Comment("comment".to_owned()),
    ] {
        assert_eq!(
            assemble(
                Some(name(None)),
                vec![Item::Child(child), namespace("p", "urn:p")],
                2,
                2,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err()
            .code,
            "XTDE0410"
        );
    }
    assert_eq!(
        assemble(
            None,
            vec![namespace("p", "urn:p")],
            1,
            1,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "XTDE0420"
    );
    let result = assemble(
        Some(name(None)),
        vec![
            Item::Child(ResultNode::Text(String::new())),
            namespace("p", "urn:p"),
        ],
        2,
        2,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(bytes(&result), "<out xmlns:p=\"urn:p\"></out>");
}

#[test]
fn default_namespace_requires_a_namespaced_destination() {
    assert_eq!(
        assemble(
            Some(name(None)),
            vec![namespace("", "urn:d")],
            1,
            1,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "XTDE0440"
    );
    let result = assemble(
        Some(name(Some("urn:d"))),
        vec![namespace("", "urn:d")],
        1,
        1,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(bytes(&result), "<out xmlns=\"urn:d\"></out>");
}

#[test]
fn copied_binding_participates_in_existing_descendant_namespace_fixup() {
    let child = ResultNode::Element {
        name: ExpandedName {
            namespace: Some("urn:p".to_owned()),
            local: "child".to_owned(),
        },
        namespaces: Vec::new().into(),
        attributes: Vec::new(),
        children: Vec::new(),
    };
    let result = assemble(
        Some(name(None)),
        vec![namespace("p", "urn:p"), Item::Child(child)],
        2,
        1,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(
        bytes(&result),
        "<out xmlns:p=\"urn:p\"><p:child></p:child></out>"
    );
}

#[test]
fn attachment_capacity_bytes_and_cancellation_fail_without_partial_result() {
    for (items, bindings) in [(0, 1), (1, 0)] {
        assert_eq!(
            assemble(
                Some(name(None)),
                vec![namespace("p", "urn:p")],
                items,
                bindings,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err()
            .category,
            FailureCategory::Limit
        );
    }
    let mut limits = WorkLimits::unbounded();
    limits.result_text_bytes = 5;
    let mut bounded = InvocationControl::new(CancellationToken::new(), limits);
    assert_eq!(
        assemble(
            Some(name(None)),
            vec![namespace("p", "urn:p")],
            1,
            1,
            &mut bounded
        )
        .unwrap_err()
        .code,
        "FXCT0002"
    );
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 1);
    assert_eq!(
        assemble(
            Some(name(None)),
            vec![namespace("p", "urn:p"), namespace("q", "urn:q")],
            2,
            2,
            &mut cancelled
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    assert_eq!(
        bytes(
            &assemble(
                Some(name(None)),
                vec![namespace("p", "urn:p")],
                1,
                1,
                &mut InvocationControl::unbounded()
            )
            .unwrap()
        ),
        "<out xmlns:p=\"urn:p\"></out>"
    );
}
