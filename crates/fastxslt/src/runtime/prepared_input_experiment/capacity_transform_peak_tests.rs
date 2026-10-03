//! Complete single-thread requested allocation scopes, not process-memory estimates.

use crate::execution_control_experiment::InvocationControl;
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{compile_resource, execute_program, serialize_xml};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::parse_document_controlled;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;
use allocation_counter::{AllocationInfo, measure};

use super::{Capacity, Fixture, IDENTITY, LIMITS, compiled, expected, fixtures};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Prepared,
    Result,
    Serialized,
    Released,
}

fn copy_program() -> StylesheetProgram {
    const ID: &str = "urn:ar0027:capacity-copy-style";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit(ID, br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:copy-of select="/"/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    compile_resource(&resources.seal(), ID).unwrap()
}

fn scope(
    program: &StylesheetProgram,
    fixture: &Fixture,
    capacity: Capacity,
    stage: Stage,
    expected: &str,
) -> AllocationInfo {
    let mut retained = None;
    // Each stage is a fresh complete prefix. Do not nest counters: this counter
    // merges nested maxima additively, which would not be a co-resident peak.
    let allocations = measure(|| {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
        resources
            .admit(IDENTITY, fixture.xml.as_bytes().to_vec())
            .unwrap();
        let snapshot = resources.seal();
        let mut control = InvocationControl::unbounded();
        let parsed = parse_document_controlled(
            IDENTITY,
            snapshot.get(IDENTITY).unwrap(),
            LIMITS,
            &mut control,
        )
        .unwrap();
        let document = match capacity {
            Capacity::Growth => Document::from_parsed_controlled(parsed, &mut control),
            Capacity::Frozen => {
                Document::from_parsed_with_frozen_node_capacity(parsed, &mut control)
            }
            Capacity::Presized => {
                Document::from_parsed_with_presized_node_capacity(parsed, &mut control)
            }
        }
        .unwrap();
        assert_eq!(document.node_count(), fixture.nodes);
        let result = (stage >= Stage::Result)
            .then(|| execute_program(program, &document, "capacity-peak", &mut control).unwrap());
        let serialized = result
            .as_ref()
            .filter(|_| stage >= Stage::Serialized)
            .map(|result| {
                let value = serialize_xml(
                    result,
                    &program.output,
                    "capacity-peak",
                    1_048_576,
                    &mut control,
                )
                .unwrap();
                assert_eq!(value, expected);
                value
            });
        if stage != Stage::Released {
            retained = Some((snapshot, document, result, serialized));
        }
    });
    if stage == Stage::Released {
        assert_eq!(allocations.count_current, 0);
        assert_eq!(allocations.bytes_current, 0);
    } else {
        assert!(allocations.bytes_current > 0);
    }
    drop(retained);
    allocations
}

fn reference_copy(program: &StylesheetProgram, fixture: &Fixture) -> String {
    let parsed = parse_document_controlled(
        IDENTITY,
        fixture.xml.as_bytes(),
        LIMITS,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = Document::from_parsed(parsed).unwrap();
    let mut control = InvocationControl::unbounded();
    let result = execute_program(program, &source, "copy-reference", &mut control).unwrap();
    drop(source);
    serialize_xml(
        &result,
        &program.output,
        "copy-reference",
        1_048_576,
        &mut control,
    )
    .unwrap()
}

fn run(print: bool) {
    let count = compiled();
    let copy = copy_program();
    for fixture in fixtures() {
        let expected_copy = if fixture.name == "deep" {
            format!("{}leaf{}", "<n>".repeat(256), "</n>".repeat(256))
        } else {
            reference_copy(&copy, &fixture)
        };
        for (name, program, expected) in [
            ("count", &count, expected(&fixture)),
            ("copy", &copy, expected_copy.as_str()),
        ] {
            for capacity in [Capacity::Growth, Capacity::Frozen, Capacity::Presized] {
                for stage in [
                    Stage::Prepared,
                    Stage::Result,
                    Stage::Serialized,
                    Stage::Released,
                ] {
                    let allocations = scope(program, &fixture, capacity, stage, expected);
                    if print {
                        println!(
                            "ar0027 full_transform_shape={} workload={} candidate={} stage={stage:?} source_bytes={} result_bytes={} allocations={allocations:?}",
                            fixture.name,
                            name,
                            capacity.name(),
                            fixture.xml.len(),
                            expected.len()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn capacity_full_transform_allocation_scopes_release_admitted_shapes() {
    run(false);
}

#[test]
#[ignore = "manual release AR-0027 full-transform requested-allocation matrix; not a timing probe"]
fn measure_capacity_full_transform_allocations() {
    run(true);
}

#[test]
fn capacity_full_transform_deep_copy_reference() {
    let fixture = fixtures()
        .into_iter()
        .find(|fixture| fixture.name == "deep")
        .unwrap();
    let program = copy_program();
    let parsed = parse_document_controlled(
        IDENTITY,
        fixture.xml.as_bytes(),
        LIMITS,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = Document::from_parsed(parsed).unwrap();
    let mut control = InvocationControl::unbounded();
    let result = execute_program(&program, &source, "deep-reference", &mut control).unwrap();
    drop(source);
    let observed = serialize_xml(
        &result,
        &program.output,
        "deep-reference",
        1_048_576,
        &mut control,
    )
    .unwrap();
    assert_eq!(
        observed,
        format!("{}leaf{}", "<n>".repeat(256), "</n>".repeat(256))
    );
}
