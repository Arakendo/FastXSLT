//! Independent node-capacity candidates against unchanged growth construction.

use std::{fmt::Write as _, sync::Arc, time::Instant};

use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::parse_document_controlled;

use super::{Fixture, IDENTITY, LIMITS, fixtures};

fn comparison_fixtures() -> Vec<Fixture> {
    let mut fixtures = fixtures();
    let mut attributes = String::new();
    for index in 0..31 {
        write!(attributes, " a{index}='same'").unwrap();
    }
    // 2 + 127 * (element + 31 attributes) = 4066 nodes, close to 4096
    // growth slots. Small retention benefit can conceal a costly final resize.
    fixtures.push(Fixture {
        name: "near-full-attributes",
        xml: format!("<r>{}</r>", format!("<item{attributes}/>").repeat(127)),
        nodes: 4066,
        text_bytes: 0,
    });
    fixtures
}

fn prepare(bytes: &[u8], frozen: bool, control: &mut InvocationControl) -> Document {
    let parsed = parse_document_controlled(IDENTITY, bytes, LIMITS, control).unwrap();
    if frozen {
        Document::from_parsed_with_frozen_node_capacity(parsed, control)
    } else {
        Document::from_parsed_controlled(parsed, control)
    }
    .unwrap()
}

fn prepare_presized(bytes: &[u8], control: &mut InvocationControl) -> Document {
    let parsed = parse_document_controlled(IDENTITY, bytes, LIMITS, control).unwrap();
    Document::from_parsed_with_presized_node_capacity(parsed, control).unwrap()
}

#[test]
fn freeze_candidate_preserves_nodes_and_charges_across_shapes() {
    for fixture in comparison_fixtures() {
        let mut reference_control = InvocationControl::unbounded();
        let reference = prepare(fixture.xml.as_bytes(), false, &mut reference_control);
        let mut frozen_control = InvocationControl::unbounded();
        let frozen = prepare(fixture.xml.as_bytes(), true, &mut frozen_control);
        assert_eq!(reference.node_count(), frozen.node_count());
        assert!(!reference.same_origin(&frozen));
        assert!(frozen.owned_capacity_bytes() <= reference.owned_capacity_bytes());
        let before = reference.capacity_anatomy();
        let after = frozen.capacity_anatomy();
        assert!(after.node_capacity >= after.node_count);
        assert_eq!(before.live_node_records, after.live_node_records);
        assert_eq!(before.child_ids, after.child_ids);
        assert_eq!(before.attribute_ids, after.attribute_ids);
        let mut pending = vec![reference.document_node()];
        while let Some(node) = pending.pop() {
            assert_eq!(reference.kind(node), frozen.kind(node));
            assert_eq!(reference.name(node), frozen.name(node));
            assert_eq!(reference.prefix(node), frozen.prefix(node));
            assert_eq!(reference.value(node), frozen.value(node));
            assert_eq!(reference.parent(node), frozen.parent(node));
            assert_eq!(reference.children(node), frozen.children(node));
            assert_eq!(reference.attributes(node), frozen.attributes(node));
            assert_eq!(
                reference.namespace_declarations(node),
                frozen.namespace_declarations(node)
            );
            assert_eq!(reference.document_order(node), frozen.document_order(node));
            assert_eq!(reference.location(node), frozen.location(node));
            pending.extend(reference.children(node));
            pending.extend(reference.attributes(node));
        }
        assert_eq!(
            reference.string_value(reference.document_node()),
            frozen.string_value(frozen.document_node())
        );
        for domain in [WorkDomain::XmlEvent, WorkDomain::XdmNode] {
            assert_eq!(
                reference_control.consumed(domain),
                frozen_control.consumed(domain)
            );
        }
        let used = reference_control.consumed(WorkDomain::XdmNode);
        for limit in [used, used - 1] {
            let parsed = parse_document_controlled(
                IDENTITY,
                fixture.xml.as_bytes(),
                LIMITS,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let mut limits = WorkLimits::unbounded();
            limits.xdm_nodes = limit;
            let reference = Document::from_parsed_controlled(
                parsed,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            let parsed = parse_document_controlled(
                IDENTITY,
                fixture.xml.as_bytes(),
                LIMITS,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let frozen = Document::from_parsed_with_frozen_node_capacity(
                parsed,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            assert_eq!(reference.as_ref().map(|_| ()), frozen.as_ref().map(|_| ()));
        }
    }
}

#[test]
fn freeze_candidate_preserves_cancelled_construction() {
    let bytes = b"<r><a/><b/></r>";
    for offset in [0, 1, 2, 3] {
        let parsed =
            parse_document_controlled(IDENTITY, bytes, LIMITS, &mut InvocationControl::unbounded())
                .unwrap();
        let reference = Document::from_parsed_controlled(
            parsed,
            &mut InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, offset),
        )
        .unwrap_err();
        let parsed =
            parse_document_controlled(IDENTITY, bytes, LIMITS, &mut InvocationControl::unbounded())
                .unwrap();
        let frozen = Document::from_parsed_with_frozen_node_capacity(
            parsed,
            &mut InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, offset),
        )
        .unwrap_err();
        assert_eq!(reference, frozen);
    }
}

#[test]
fn presizing_preserves_coalesced_text_counts_and_budget_failures() {
    let mut inputs: Vec<_> = comparison_fixtures()
        .into_iter()
        .map(|fixture| fixture.xml.into_bytes())
        .collect();
    inputs.push(b"<r>a<![CDATA[b]]>c<x/>d<![CDATA[e]]><!--break-->f</r>".to_vec());
    for bytes in inputs {
        let reference = prepare(&bytes, false, &mut InvocationControl::unbounded());
        let used = reference.node_count();
        for limit in [used, used - 1] {
            let mut limits = WorkLimits::unbounded();
            limits.xdm_nodes = limit;
            let parsed = parse_document_controlled(
                IDENTITY,
                &bytes,
                LIMITS,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let mut reference_control = InvocationControl::new(CancellationToken::new(), limits);
            let ordinary = Document::from_parsed_controlled(parsed, &mut reference_control);
            let parsed = parse_document_controlled(
                IDENTITY,
                &bytes,
                LIMITS,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let mut candidate_control = InvocationControl::new(CancellationToken::new(), limits);
            let candidate =
                Document::from_parsed_with_presized_node_capacity(parsed, &mut candidate_control);
            assert_eq!(
                ordinary.as_ref().map(|_| ()),
                candidate.as_ref().map(|_| ())
            );
            assert_eq!(
                reference_control.consumed(WorkDomain::XdmNode),
                candidate_control.consumed(WorkDomain::XdmNode)
            );
            if let Ok(candidate) = candidate {
                assert_eq!(candidate.node_count(), used);
                assert_eq!(candidate.capacity_anatomy().node_capacity, used);
                assert_eq!(
                    candidate.string_value(candidate.document_node()),
                    reference.string_value(reference.document_node())
                );
                let mut pending = vec![reference.document_node()];
                while let Some(node) = pending.pop() {
                    assert_eq!(candidate.kind(node), reference.kind(node));
                    assert_eq!(candidate.name(node), reference.name(node));
                    assert_eq!(candidate.prefix(node), reference.prefix(node));
                    assert_eq!(candidate.value(node), reference.value(node));
                    assert_eq!(candidate.parent(node), reference.parent(node));
                    assert_eq!(
                        candidate.namespace_declarations(node),
                        reference.namespace_declarations(node)
                    );
                    assert_eq!(candidate.location(node), reference.location(node));
                    assert_eq!(candidate.children(node), reference.children(node));
                    assert_eq!(candidate.attributes(node), reference.attributes(node));
                    assert_eq!(
                        candidate.document_order(node),
                        reference.document_order(node)
                    );
                    pending.extend(reference.children(node));
                    pending.extend(reference.attributes(node));
                }
            }
        }
        for offset in [0, 1, used - 1] {
            let parsed = parse_document_controlled(
                IDENTITY,
                &bytes,
                LIMITS,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let ordinary = Document::from_parsed_controlled(
                parsed,
                &mut InvocationControl::unbounded()
                    .cancelling_on_charge(WorkDomain::XdmNode, offset),
            )
            .unwrap_err();
            let parsed = parse_document_controlled(
                IDENTITY,
                &bytes,
                LIMITS,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let candidate = Document::from_parsed_with_presized_node_capacity(
                parsed,
                &mut InvocationControl::unbounded()
                    .cancelling_on_charge(WorkDomain::XdmNode, offset),
            )
            .unwrap_err();
            assert_eq!(ordinary, candidate);
        }
    }
}

#[test]
fn freeze_candidate_preserves_pinned_transform_result() {
    use crate::resources::{ResourceLimits, ResourceSetBuilder};
    use crate::runtime::golden_runtime_experiment::{
        compile_resource, execute_program, serialize_xml,
    };

    const STYLE_ID: &str = "urn:ar0027:freeze-style";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources
        .admit(
            STYLE_ID,
            include_bytes!("../../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
        )
        .unwrap();
    let program = compile_resource(&resources.seal(), STYLE_ID).unwrap();
    let bytes = include_bytes!("../../../../../vendor/xslt30-test/tests/expr/for/for03.xml");
    for candidate in 0..3 {
        let mut preparation = InvocationControl::unbounded();
        let document = if candidate == 2 {
            prepare_presized(bytes, &mut preparation)
        } else {
            prepare(bytes, candidate == 1, &mut preparation)
        };
        let mut control = InvocationControl::unbounded();
        let result = execute_program(&program, &document, "ar0027-freeze", &mut control).unwrap();
        assert_eq!(
            serialize_xml(
                &result,
                &program.output,
                "ar0027-freeze",
                8192,
                &mut control
            )
            .unwrap(),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>"
        );
    }
}

#[test]
#[ignore = "manual release freeze-only preparation comparison; omit allocator feature"]
fn measure_node_freeze_latency() {
    fn median(samples: &[(u128, u128)], release: bool) -> u128 {
        let mut values: Vec<_> = samples
            .iter()
            .map(|sample| if release { sample.1 } else { sample.0 })
            .collect();
        values.sort_unstable();
        values[500]
    }
    for fixture in comparison_fixtures() {
        let mut reference = Vec::with_capacity(1001);
        let mut frozen = Vec::with_capacity(1001);
        for index in 0..1033 {
            for candidate in [index % 2 == 0, index % 2 != 0] {
                let mut control = InvocationControl::unbounded();
                let start = Instant::now();
                let document = Arc::new(prepare(fixture.xml.as_bytes(), candidate, &mut control));
                let elapsed = start.elapsed().as_nanos();
                assert_eq!(document.node_count(), fixture.nodes);
                drop(document);
                let released = start.elapsed().as_nanos();
                if index >= 32 {
                    if candidate {
                        frozen.push((elapsed, released));
                    } else {
                        reference.push((elapsed, released));
                    }
                }
            }
        }
        println!(
            "ar0027 freeze_shape={} reference_median_ns={} frozen_median_ns={} reference_prepare_release_ns={} frozen_prepare_release_ns={}",
            fixture.name,
            median(&reference, false),
            median(&frozen, false),
            median(&reference, true),
            median(&frozen, true)
        );
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual release freeze-only complete preparation allocation comparison"]
fn measure_node_freeze_allocations() {
    for fixture in comparison_fixtures() {
        for frozen in [false, true] {
            let mut document = None;
            let allocations = allocation_counter::measure(|| {
                document = Some(Arc::new(prepare(
                    fixture.xml.as_bytes(),
                    frozen,
                    &mut InvocationControl::unbounded(),
                )));
            });
            let document = document.unwrap();
            println!(
                "ar0027 freeze_shape={} frozen={frozen} capacity={} slots={} allocations={allocations:?}",
                fixture.name,
                document.owned_capacity_bytes(),
                document.capacity_anatomy().node_capacity
            );
        }
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual release presized complete preparation allocation comparison"]
fn measure_node_presized_allocations() {
    for fixture in comparison_fixtures() {
        for presized in [false, true] {
            let mut document = None;
            let allocations = allocation_counter::measure(|| {
                let mut control = InvocationControl::unbounded();
                document = Some(Arc::new(if presized {
                    prepare_presized(fixture.xml.as_bytes(), &mut control)
                } else {
                    prepare(fixture.xml.as_bytes(), false, &mut control)
                }));
            });
            let document = document.unwrap();
            println!(
                "ar0027 presized_shape={} presized={presized} capacity={} slots={} allocations={allocations:?}",
                fixture.name,
                document.owned_capacity_bytes(),
                document.capacity_anatomy().node_capacity
            );
        }
    }
}

#[test]
#[ignore = "manual release presized preparation comparison; omit allocator feature"]
fn measure_node_presized_latency() {
    fn median(samples: &[(u128, u128)], release: bool) -> u128 {
        let mut values: Vec<_> = samples
            .iter()
            .map(|sample| if release { sample.1 } else { sample.0 })
            .collect();
        values.sort_unstable();
        values[500]
    }
    for fixture in comparison_fixtures() {
        let mut reference = Vec::with_capacity(1001);
        let mut presized = Vec::with_capacity(1001);
        for index in 0..1033 {
            for candidate in [index % 2 == 0, index % 2 != 0] {
                let mut control = InvocationControl::unbounded();
                let start = Instant::now();
                let document = Arc::new(if candidate {
                    prepare_presized(fixture.xml.as_bytes(), &mut control)
                } else {
                    prepare(fixture.xml.as_bytes(), false, &mut control)
                });
                let elapsed = start.elapsed().as_nanos();
                assert_eq!(document.node_count(), fixture.nodes);
                drop(document);
                let released = start.elapsed().as_nanos();
                if index >= 32 {
                    if candidate {
                        presized.push((elapsed, released));
                    } else {
                        reference.push((elapsed, released));
                    }
                }
            }
        }
        println!(
            "ar0027 presized_shape={} reference_median_ns={} presized_median_ns={} reference_prepare_release_ns={} presized_prepare_release_ns={}",
            fixture.name,
            median(&reference, false),
            median(&presized, false),
            median(&reference, true),
            median(&presized, true)
        );
    }
}
