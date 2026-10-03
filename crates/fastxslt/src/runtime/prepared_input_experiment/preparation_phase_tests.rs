//! AR-0027 attribution at the existing controlled preparation seam.

use std::{hint::black_box, time::Instant};

use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder, ResourceSnapshot};
use crate::runtime::golden_runtime_experiment::{compile_resource, execute_program, serialize_xml};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::ParseLimits;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::{PreparationPhaseObservation, prepare_document, prepare_document_observed};

const SOURCE_ID: &str = "urn:ar0027:phase-source";
const STYLE_ID: &str = "urn:ar0027:phase-style";
const LIMITS: ParseLimits = ParseLimits {
    max_events: 100_000,
    max_depth: 64,
};

fn fixtures() -> Vec<(&'static str, Vec<u8>, String)> {
    let mut fixtures = vec![(
        "pinned-for004",
        include_bytes!("../../../../../vendor/xslt30-test/tests/expr/for/for03.xml").to_vec(),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>".to_owned(),
    )];
    for (name, items) in [("wide-5", 5), ("wide-50", 50), ("wide-500", 500)] {
        let source = format!(
            "<order>{}</order>",
            "<order-item price=\"1\" qty=\"1\"/>".repeat(items)
        );
        fixtures.push((
            name,
            source.into_bytes(),
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{items}.00</out>"),
        ));
    }
    fixtures
}

fn snapshot(identity: &str, bytes: &[u8]) -> ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
    builder.admit(identity, bytes.to_vec()).unwrap();
    builder.seal()
}

fn program() -> StylesheetProgram {
    compile_resource(
        &snapshot(
            STYLE_ID,
            include_bytes!("../../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl"),
        ),
        STYLE_ID,
    )
    .unwrap()
}

fn assert_result(program: &StylesheetProgram, document: &Document, expected: &str) {
    let mut control = InvocationControl::unbounded();
    let result = execute_program(program, document, "ar0027-phases", &mut control).unwrap();
    assert_eq!(
        serialize_xml(
            &result,
            &program.output,
            "ar0027-phases",
            8192,
            &mut control
        )
        .unwrap(),
        expected
    );
}

#[test]
fn phase_observation_preserves_tree_provenance_and_work_charges() {
    let program = program();
    for (_, source, expected) in fixtures() {
        let snapshot = snapshot(SOURCE_ID, &source);
        let mut reference_control = InvocationControl::unbounded();
        let (reference, reference_capacity) =
            prepare_document(&snapshot, LIMITS, SOURCE_ID, &mut reference_control).unwrap();
        let mut observed_control = InvocationControl::unbounded();
        let (observed, observed_capacity) = prepare_document_observed(
            &snapshot,
            LIMITS,
            SOURCE_ID,
            &mut observed_control,
            &mut PreparationPhaseObservation::default(),
        )
        .unwrap();
        assert!(!reference.same_origin(&observed));
        assert_eq!(reference.node_count(), observed.node_count());
        assert_eq!(reference_capacity, observed_capacity);
        assert_eq!(
            reference.owned_capacity_bytes(),
            observed.owned_capacity_bytes()
        );
        let mut pending = vec![reference.document_node()];
        while let Some(node) = pending.pop() {
            assert_eq!(reference.kind(node), observed.kind(node));
            assert_eq!(reference.name(node), observed.name(node));
            assert_eq!(reference.prefix(node), observed.prefix(node));
            assert_eq!(reference.value(node), observed.value(node));
            assert_eq!(reference.parent(node), observed.parent(node));
            assert_eq!(reference.children(node), observed.children(node));
            assert_eq!(reference.attributes(node), observed.attributes(node));
            assert_eq!(
                reference.namespace_declarations(node),
                observed.namespace_declarations(node)
            );
            assert_eq!(
                reference.document_order(node),
                observed.document_order(node)
            );
            assert_eq!(reference.location(node), observed.location(node));
            pending.extend(reference.children(node));
            pending.extend(reference.attributes(node));
        }
        for domain in [WorkDomain::XmlEvent, WorkDomain::XdmNode] {
            assert_eq!(
                reference_control.consumed(domain),
                observed_control.consumed(domain)
            );
            // Exact preparation ceilings succeed; one-less fails identically.
            let used = reference_control.consumed(domain);
            for limit in [used, used - 1] {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::XmlEvent => limits.xml_events = limit,
                    WorkDomain::XdmNode => limits.xdm_nodes = limit,
                    _ => unreachable!(),
                }
                let ordinary = prepare_document(
                    &snapshot,
                    LIMITS,
                    SOURCE_ID,
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                );
                let observed = prepare_document_observed(
                    &snapshot,
                    LIMITS,
                    SOURCE_ID,
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                    &mut PreparationPhaseObservation::default(),
                );
                assert_eq!(ordinary.as_ref().map(|_| ()), observed.as_ref().map(|_| ()));
                assert_eq!(ordinary.is_ok(), limit == used);
            }
        }
        assert_result(&program, &reference, &expected);
        assert_result(&program, &observed, &expected);
    }
}

#[test]
fn phase_observation_preserves_cancellation_and_invalid_xml_failures() {
    let snapshot = snapshot(SOURCE_ID, b"<r><a/></r>");
    for domain in [WorkDomain::XmlEvent, WorkDomain::XdmNode] {
        let ordinary = prepare_document(
            &snapshot,
            LIMITS,
            SOURCE_ID,
            &mut InvocationControl::unbounded().cancelling_on_charge(domain, 2),
        )
        .unwrap_err();
        let observed = prepare_document_observed(
            &snapshot,
            LIMITS,
            SOURCE_ID,
            &mut InvocationControl::unbounded().cancelling_on_charge(domain, 2),
            &mut PreparationPhaseObservation::default(),
        )
        .unwrap_err();
        assert_eq!(ordinary, observed);
    }
    let malformed = self::snapshot(SOURCE_ID, b"<r><a></r>");
    for identity in [SOURCE_ID, "urn:ar0027:missing"] {
        let ordinary = prepare_document(
            &malformed,
            LIMITS,
            identity,
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        let observed = prepare_document_observed(
            &malformed,
            LIMITS,
            identity,
            &mut InvocationControl::unbounded(),
            &mut PreparationPhaseObservation::default(),
        )
        .unwrap_err();
        assert_eq!(ordinary, observed);
    }
}

#[derive(Default)]
struct Sample {
    xml_parse: u128,
    xdm_build: u128,
    preparation: u128,
}

fn sample(
    snapshot: &ResourceSnapshot,
    program: &StylesheetProgram,
    expected: &str,
    observed: bool,
) -> Sample {
    let mut control = InvocationControl::unbounded();
    let mut phases = PreparationPhaseObservation::default();
    let start = Instant::now();
    let (document, _) = if observed {
        prepare_document_observed(snapshot, LIMITS, SOURCE_ID, &mut control, &mut phases)
    } else {
        prepare_document(snapshot, LIMITS, SOURCE_ID, &mut control)
    }
    .unwrap();
    let preparation_ns = start.elapsed().as_nanos();
    // Result validation and document release are outside preparation clocks.
    assert_result(program, &document, expected);
    black_box(&document);
    Sample {
        xml_parse: phases.xml_parse.as_nanos(),
        xdm_build: phases.xdm_build.as_nanos(),
        preparation: preparation_ns,
    }
}

fn median(samples: &[Sample], field: fn(&Sample) -> u128) -> u128 {
    let mut values: Vec<_> = samples.iter().map(field).collect();
    values.sort_unstable();
    values[values.len() / 2]
}

#[test]
#[ignore = "manual release shared-preparation phase attribution; no layout change"]
fn measure_shared_preparation_phases() {
    const SAMPLES: usize = 1001;
    let program = program();
    for (name, source, expected) in fixtures() {
        let snapshot = snapshot(SOURCE_ID, &source);
        for _ in 0..32 {
            sample(&snapshot, &program, &expected, false);
            sample(&snapshot, &program, &expected, true);
        }
        let mut ordinary = Vec::with_capacity(SAMPLES);
        let mut observed = Vec::with_capacity(SAMPLES);
        for index in 0..SAMPLES {
            if index % 2 == 0 {
                ordinary.push(sample(&snapshot, &program, &expected, false));
                observed.push(sample(&snapshot, &program, &expected, true));
            } else {
                observed.push(sample(&snapshot, &program, &expected, true));
                ordinary.push(sample(&snapshot, &program, &expected, false));
            }
        }
        println!(
            "ar0027 fixture={name} samples={SAMPLES} bytes={} xml_median_ns={} xdm_median_ns={} instrumented_preparation_median_ns={} ordinary_preparation_median_ns={}",
            source.len(),
            median(&observed, |sample| sample.xml_parse),
            median(&observed, |sample| sample.xdm_build),
            median(&observed, |sample| sample.preparation),
            median(&ordinary, |sample| sample.preparation)
        );
    }
}
