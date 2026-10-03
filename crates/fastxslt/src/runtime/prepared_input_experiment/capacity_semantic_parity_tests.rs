//! AR-0027 capacity candidates through effective semantics and owner lifetimes.

use std::sync::{Arc, Barrier};

use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{compile_resource, execute_program, serialize_xml};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xdm::qualified_nodes::QualifiedSourceNode;
use crate::xml::internal_subset::InternalSubsetLimits;
use crate::xml::quick_xml_experiment::{
    ParseLimits, parse_document_controlled_with_internal_subset,
};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

const SOURCE_ID: &str = "urn:ar0027:capacity-parity-source";
const SOURCE: &[u8] = br#"<!DOCTYPE r [<!ATTLIST item key ID #IMPLIED note CDATA "default">]><r xmlns:p="urn:outer"> <item key="alpha"> <![CDATA[value]]><!--c--><?pi test?><p:leaf xmlns:p="urn:inner"/> </item> <item key="beta" xml:space="preserve"> <a xmlns="urn:default"><b xmlns=""/></a> </item> </r>"#;
const DOMAINS: [WorkDomain; 10] = [
    WorkDomain::XmlEvent,
    WorkDomain::XdmNode,
    WorkDomain::XdmStringValueNode,
    WorkDomain::XPathNodeVisit,
    WorkDomain::XPathOperation,
    WorkDomain::XsltInstruction,
    WorkDomain::XsltTemplateCandidate,
    WorkDomain::ResultNode,
    WorkDomain::ResultTextByte,
    WorkDomain::SerializedByte,
];

#[derive(Clone, Copy)]
enum Capacity {
    Growth,
    Frozen,
    Presized,
}

fn prepare(capacity: Capacity) -> Document {
    let mut control = InvocationControl::unbounded();
    let parsed = parse_document_controlled_with_internal_subset(
        SOURCE_ID,
        SOURCE,
        ParseLimits {
            max_events: 1000,
            max_depth: 32,
        },
        InternalSubsetLimits {
            declarations: 16,
            nesting_depth: 8,
            references: 100,
            replacement_bytes: 8192,
        },
        &mut control,
    )
    .unwrap();
    match capacity {
        Capacity::Growth => Document::from_parsed_controlled(parsed, &mut control),
        Capacity::Frozen => Document::from_parsed_with_frozen_node_capacity(parsed, &mut control),
        Capacity::Presized => {
            Document::from_parsed_with_presized_node_capacity(parsed, &mut control)
        }
    }
    .unwrap()
}

fn program(identity: &str, strip: bool, body: &str) -> Arc<StylesheetProgram> {
    let whitespace = if strip {
        "<xsl:strip-space elements='*'/>"
    } else {
        ""
    };
    let xml = format!(
        "<xsl:stylesheet version='1.0' xmlns:xsl='http://www.w3.org/1999/XSL/Transform'><xsl:output omit-xml-declaration='yes'/>{whitespace}<xsl:template match='/'>{body}</xsl:template></xsl:stylesheet>"
    );
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit(identity, xml.into_bytes()).unwrap();
    Arc::new(compile_resource(&resources.seal(), identity).unwrap())
}

fn output(program: &StylesheetProgram, source: &Document) -> (String, Vec<usize>) {
    let mut control = InvocationControl::unbounded();
    let result = execute_program(program, source, "ar0027-parity", &mut control).unwrap();
    let xml = serialize_xml(
        &result,
        &program.output,
        "ar0027-parity",
        8192,
        &mut control,
    )
    .unwrap();
    (xml, DOMAINS.map(|domain| control.consumed(domain)).to_vec())
}

fn assert_visible_parity(reference: &Document, candidate: &Document) {
    assert_eq!(reference.node_count(), candidate.node_count());
    let mut pending = vec![reference.document_node()];
    while let Some(node) = pending.pop() {
        assert_eq!(reference.kind(node), candidate.kind(node));
        assert_eq!(reference.name(node), candidate.name(node));
        assert_eq!(reference.prefix(node), candidate.prefix(node));
        assert_eq!(reference.parent(node), candidate.parent(node));
        assert_eq!(reference.value(node), candidate.value(node));
        assert_eq!(reference.children(node), candidate.children(node));
        assert_eq!(reference.attributes(node), candidate.attributes(node));
        assert_eq!(reference.location(node), candidate.location(node));
        assert_eq!(
            reference.document_order(node),
            candidate.document_order(node)
        );
        assert_eq!(reference.string_value(node), candidate.string_value(node));
        assert_eq!(
            reference.namespace_declarations(node),
            candidate.namespace_declarations(node)
        );
        if reference.kind(node) == NodeKind::Element {
            assert_namespaces(reference, candidate, node);
        }
        pending.extend(reference.children(node));
        pending.extend(reference.attributes(node));
    }
    for id in ["alpha", "beta", "absent"] {
        assert_eq!(
            reference.elements_with_id(id),
            candidate.elements_with_id(id)
        );
    }
}

fn assert_namespaces(reference: &Document, candidate: &Document, element: NodeId) {
    let mut left_control = InvocationControl::unbounded();
    let mut right_control = InvocationControl::unbounded();
    let left = reference
        .namespace_nodes_controlled(element, 16, &mut left_control)
        .unwrap();
    let right = candidate
        .namespace_nodes_controlled(element, 16, &mut right_control)
        .unwrap();
    assert_eq!(left.len(), right.len());
    for (left, right) in left.into_iter().zip(right) {
        let a = left.namespace().unwrap();
        let b = right.namespace().unwrap();
        assert_eq!(
            (a.prefix(), a.string_value(), a.declaration()),
            (b.prefix(), b.string_value(), b.declaration())
        );
        assert!(left.parent().unwrap().same_node(QualifiedSourceNode::Tree {
            document: reference,
            id: element
        }));
        assert_eq!(left.same_node(right), reference.same_origin(candidate));
        assert_eq!(
            left.compare_same_origin(right).is_some(),
            reference.same_origin(candidate)
        );
    }
    assert_eq!(
        left_control.consumed(WorkDomain::XdmNode),
        right_control.consumed(WorkDomain::XdmNode)
    );
    for ceiling in [0, 1] {
        assert_eq!(
            reference
                .namespace_nodes_controlled(element, ceiling, &mut InvocationControl::unbounded())
                .as_ref()
                .map(|_| ()),
            candidate
                .namespace_nodes_controlled(element, ceiling, &mut InvocationControl::unbounded())
                .as_ref()
                .map(|_| ())
        );
    }
}

#[test]
fn capacity_candidates_preserve_typed_ids_effective_views_and_namespaces() {
    let reference = prepare(Capacity::Growth);
    assert_eq!(reference.elements_with_id("alpha").len(), 1);
    let derived = reference
        .derive_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
        .unwrap();
    for capacity in [Capacity::Frozen, Capacity::Presized] {
        let candidate = prepare(capacity);
        assert!(!reference.same_origin(&candidate));
        assert_visible_parity(&reference, &candidate);
        let mut control = InvocationControl::unbounded();
        let view = candidate
            .view_stripping_all_element_whitespace(&mut control)
            .unwrap();
        assert!(view.shares_node_storage_with(&candidate));
        assert!(view.same_origin(&candidate));
        let preserved = candidate.elements_with_id("beta")[0];
        assert_eq!(view.children(preserved).len(), 3);
        assert_visible_parity(&derived, &view);
        for element in candidate
            .elements_with_id("alpha")
            .iter()
            .chain(candidate.elements_with_id("beta"))
        {
            assert_namespaces(&candidate, &view, *element);
        }
        let used = control.consumed(WorkDomain::XdmNode);
        for limit in [used, used - 1] {
            let mut limits = WorkLimits::unbounded();
            limits.xdm_nodes = limit;
            let mut a = InvocationControl::new(CancellationToken::new(), limits);
            let mut b = InvocationControl::new(CancellationToken::new(), limits);
            assert_eq!(
                reference
                    .view_stripping_all_element_whitespace(&mut a)
                    .as_ref()
                    .map(|_| ()),
                candidate
                    .view_stripping_all_element_whitespace(&mut b)
                    .as_ref()
                    .map(|_| ())
            );
        }
        for offset in [0, used - 1] {
            let mut a =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, offset);
            let mut b =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, offset);
            assert_eq!(
                reference
                    .view_stripping_all_element_whitespace(&mut a)
                    .unwrap_err(),
                candidate
                    .view_stripping_all_element_whitespace(&mut b)
                    .unwrap_err()
            );
        }
    }
}

#[test]
fn capacity_candidates_preserve_transform_copy_and_failure_results() {
    let reference = prepare(Capacity::Growth);
    for strip in [false, true] {
        let compiled = program(
            "urn:ar0027:parity-style",
            strip,
            "<out><xsl:value-of select=\"id('alpha')/@key\"/><xsl:text>|</xsl:text><xsl:value-of select='count(/r/text())'/></out>",
        );
        let expected = output(&compiled, &reference);
        assert_eq!(
            expected.0,
            if strip {
                "<out>alpha|0</out>"
            } else {
                "<out>alpha|3</out>"
            }
        );
        for capacity in [Capacity::Frozen, Capacity::Presized] {
            let candidate = prepare(capacity);
            assert_eq!(output(&compiled, &candidate), expected);
            let mut limits = WorkLimits::unbounded();
            limits.xpath_node_visits = 0;
            for cancelled in [false, true] {
                let controlled = || {
                    if cancelled {
                        InvocationControl::unbounded()
                            .cancelling_on_charge(WorkDomain::XsltInstruction, 0)
                    } else {
                        InvocationControl::new(CancellationToken::new(), limits)
                    }
                };
                assert_eq!(
                    execute_program(&compiled, &reference, "ar0027-parity", &mut controlled())
                        .unwrap_err(),
                    execute_program(&compiled, &candidate, "ar0027-parity", &mut controlled())
                        .unwrap_err()
                );
            }
            assert_eq!(output(&compiled, &candidate), expected);
        }
    }
    let copying = program(
        "urn:ar0027:copy-style",
        true,
        "<out><xsl:copy-of select='/r/item[1]'/></out>",
    );
    let expected = output(&copying, &reference).0;
    assert!(expected.contains("note=\"default\""));
    assert!(expected.contains("urn:inner"));
    for capacity in [Capacity::Frozen, Capacity::Presized] {
        let candidate = prepare(capacity);
        let candidate_program = program(
            "urn:ar0027:retired-copy-style",
            true,
            "<out><xsl:copy-of select='/r/item[1]'/></out>",
        );
        let mut control = InvocationControl::unbounded();
        let result = execute_program(
            &candidate_program,
            &candidate,
            "ar0027-parity",
            &mut control,
        )
        .unwrap();
        drop(candidate);
        drop(candidate_program);
        assert_eq!(
            serialize_xml(
                &result,
                &copying.output,
                "ar0027-parity",
                8192,
                &mut control
            )
            .unwrap(),
            expected
        );
    }
    let terminating = program(
        "urn:ar0027:failure-style",
        true,
        "<xsl:message terminate='yes'>stop</xsl:message>",
    );
    for capacity in [Capacity::Frozen, Capacity::Presized] {
        let candidate = prepare(capacity);
        let left = execute_program(
            &terminating,
            &reference,
            "ar0027-parity",
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        let right = execute_program(
            &terminating,
            &candidate,
            "ar0027-parity",
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert_eq!(left, right);
    }
}

#[test]
fn presized_input_concurrent_reuse_survives_generation_owner_retirement() {
    const BODY: &str = "<out><xsl:value-of select=\"id('alpha')/@key\"/><xsl:text>|</xsl:text><xsl:value-of select='count(/r/text())'/></out>";
    let old_source = Arc::new(prepare(Capacity::Presized));
    let old = program("urn:ar0027:old-style", true, BODY);
    let preserving = program("urn:ar0027:old-preserving-style", false, BODY);
    let new_source = Arc::new(prepare(Capacity::Presized));
    let new = program("urn:ar0027:new-style", false, BODY);
    assert!(!old_source.same_origin(&new_source));
    let node = old_source.elements_with_id("alpha")[0];
    assert!(
        !QualifiedSourceNode::Tree {
            document: &old_source,
            id: node
        }
        .same_node(QualifiedSourceNode::Tree {
            document: &new_source,
            id: node
        })
    );
    let old_source_weak = Arc::downgrade(&old_source);
    let old_program_weak = Arc::downgrade(&old);
    let barrier = Arc::new(Barrier::new(5));
    let workers: Vec<_> = (0..4)
        .map(|index| {
            let source = old_source.clone();
            let compiled = if index % 2 == 0 {
                old.clone()
            } else {
                preserving.clone()
            };
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                for _ in 0..32 {
                    assert_eq!(
                        output(&compiled, &source).0,
                        if index % 2 == 0 {
                            "<out>alpha|0</out>"
                        } else {
                            "<out>alpha|3</out>"
                        }
                    );
                }
            })
        })
        .collect();
    drop(old_source);
    drop(old);
    drop(preserving);
    assert!(old_source_weak.upgrade().is_some());
    assert!(old_program_weak.upgrade().is_some());
    barrier.wait();
    for _ in 0..32 {
        assert_eq!(output(&new, &new_source).0, "<out>alpha|3</out>");
    }
    for worker in workers {
        worker.join().unwrap();
    }
    assert!(old_source_weak.upgrade().is_none());
    assert!(old_program_weak.upgrade().is_none());
    assert_eq!(output(&new, &new_source).0, "<out>alpha|3</out>");
}
