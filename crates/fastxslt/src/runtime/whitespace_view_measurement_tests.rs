//! Manual AR-0016 complete-reference versus visibility-view measurements.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::hint::black_box;
use std::thread;
use std::time::Instant;

use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

use super::{MultipleMatchPolicy, WhitespaceRepresentation, execute_program_with_parameters_using};

struct MeasurementWorkload {
    name: &'static str,
    source: Document,
    iterations: usize,
}

fn compile_measurement_program(
    strip: bool,
) -> crate::xslt::golden_semantics_experiment::StylesheetProgram {
    let declaration = if strip {
        r#"<xsl:strip-space elements="*"/>"#
    } else {
        ""
    };
    let stylesheet = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">{declaration}<xsl:template match="/"><out><xsl:value-of select="."/></out></xsl:template></xsl:stylesheet>"#
    );
    let stylesheet = Document::from_parsed(
        parse_document(
            "memory:whitespace-measurement.xsl",
            stylesheet.as_bytes(),
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("measurement stylesheet should parse"),
    )
    .expect("measurement stylesheet XDM should build");
    crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("measurement stylesheet should compile")
}

fn measurement_source(name: &'static str, xml: &str, iterations: usize) -> MeasurementWorkload {
    let max_events = xml.len().saturating_mul(2).max(64);
    let identity = format!("memory:{name}.xml");
    let source = Document::from_parsed(
        parse_document(
            &identity,
            xml.as_bytes(),
            ParseLimits {
                max_events,
                max_depth: 64,
            },
        )
        .expect("measurement source should parse"),
    )
    .expect("measurement source should prepare");
    MeasurementWorkload {
        name,
        source,
        iterations,
    }
}

fn item_source(items: usize, whitespace: bool) -> String {
    let mut xml = String::from("<root>");
    for index in 0..items {
        if whitespace {
            xml.push_str("\n  ");
        }
        write!(xml, "<item>{index}</item>").expect("writing into a String cannot fail");
    }
    if whitespace {
        xml.push('\n');
    }
    xml.push_str("</root>");
    xml
}

fn deep_source(depth: usize) -> String {
    let mut xml = String::from("<root>\n");
    for _ in 0..depth {
        xml.push_str(" <level>\n");
    }
    xml.push_str(" <leaf>value</leaf>\n");
    for _ in 0..depth {
        xml.push_str(" </level>\n");
    }
    xml.push_str("</root>");
    xml
}

fn run_transform(
    program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
    source: &Document,
    representation: WhitespaceRepresentation,
    request_id: &str,
) -> super::SemanticResult {
    let mut control = InvocationControl::unbounded();
    execute_program_with_parameters_using(
        program,
        source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        request_id,
        representation,
        None,
        None,
        &mut control,
    )
    .expect("measurement transform should execute")
}

fn median_batched_ns(iterations: usize, mut operation: impl FnMut()) -> f64 {
    const SAMPLES: usize = 7;
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        let iterations = u32::try_from(iterations).expect("measurement iterations fit u32");
        samples.push(started.elapsed().as_secs_f64() * 1_000_000_000.0 / f64::from(iterations));
    }
    samples.sort_by(f64::total_cmp);
    samples[SAMPLES / 2]
}

fn latency_quantiles_ns(mut operation: impl FnMut()) -> (f64, f64, f64) {
    const OBSERVATIONS: usize = 1_001;
    let mut samples = Vec::with_capacity(OBSERVATIONS);
    for _ in 0..OBSERVATIONS {
        let started = Instant::now();
        operation();
        samples.push(started.elapsed().as_secs_f64() * 1_000_000_000.0);
    }
    samples.sort_by(f64::total_cmp);
    (samples[500], samples[950], samples[990])
}

fn concurrent_throughput(
    program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
    source: &Document,
    representation: WhitespaceRepresentation,
    iterations_per_worker: usize,
) -> f64 {
    const WORKERS: usize = 4;
    let started = Instant::now();
    thread::scope(|scope| {
        for worker in 0..WORKERS {
            scope.spawn(move || {
                for _ in 0..iterations_per_worker {
                    black_box(run_transform(
                        program,
                        source,
                        representation,
                        black_box(if worker == 0 { "worker-0" } else { "worker-n" }),
                    ));
                }
            });
        }
    });
    let operations = u32::try_from(WORKERS * iterations_per_worker)
        .expect("measurement operation count fits u32");
    f64::from(operations) / started.elapsed().as_secs_f64()
}

#[test]
#[ignore = "manual release-mode complete-reference versus visibility-view probe"]
fn measures_whitespace_reference_against_visibility_view() {
    const ITEMS: usize = 500;
    const ITERATIONS: usize = 2_000;
    const ITERATIONS_F64: f64 = 2_000.0;
    const SAMPLES: usize = 7;
    let mut source_xml = String::from("<root>\n");
    for index in 0..ITEMS {
        writeln!(source_xml, "  <item>{index}</item>").expect("writing into a String cannot fail");
    }
    source_xml.push_str("\n</root>");
    let source = Document::from_parsed(
        parse_document(
            "memory:whitespace-measurement.xml",
            source_xml.as_bytes(),
            ParseLimits {
                max_events: 4_096,
                max_depth: 8,
            },
        )
        .expect("measurement source should parse"),
    )
    .expect("measurement source should prepare");
    let stylesheet = Document::from_parsed(
        parse_document(
            "memory:whitespace-measurement.xsl",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:strip-space elements="*"/><xsl:template match="/"><out><xsl:value-of select="."/></out></xsl:template></xsl:stylesheet>"#,
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("measurement stylesheet should parse"),
    )
    .expect("measurement stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("measurement stylesheet should compile");

    let run = |representation, request_id| {
        let mut control = InvocationControl::unbounded();
        execute_program_with_parameters_using(
            &program,
            &source,
            &BTreeMap::new(),
            MultipleMatchPolicy::UseLast,
            request_id,
            representation,
            None,
            None,
            &mut control,
        )
        .expect("measurement transform should execute")
    };
    assert_eq!(
        run(WhitespaceRepresentation::CompleteReference, "reference"),
        run(WhitespaceRepresentation::VisibilityView, "view")
    );

    let mut reference_control = InvocationControl::unbounded();
    let reference = source
        .derive_stripping_all_element_whitespace(&mut reference_control)
        .expect("measurement reference should derive");
    let mut view_control = InvocationControl::unbounded();
    let view = source
        .view_stripping_all_element_whitespace(&mut view_control)
        .expect("measurement view should derive");
    let reference_bytes = reference.owned_capacity_bytes();
    let view_bytes = view.exclusive_view_capacity_bytes();

    let mut reference_ns = Vec::with_capacity(SAMPLES);
    let mut view_ns = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        for iteration in 0..ITERATIONS {
            black_box(run(
                WhitespaceRepresentation::CompleteReference,
                black_box("reference-measurement"),
            ));
            black_box(iteration);
        }
        reference_ns.push(started.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);

        let started = Instant::now();
        for iteration in 0..ITERATIONS {
            black_box(run(
                WhitespaceRepresentation::VisibilityView,
                black_box("view-measurement"),
            ));
            black_box(iteration);
        }
        view_ns.push(started.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);
    }
    reference_ns.sort_by(f64::total_cmp);
    view_ns.sort_by(f64::total_cmp);
    println!(
        "items={ITEMS} iterations={ITERATIONS} samples={SAMPLES} reference_median_ns={:.1} view_median_ns={:.1} speedup={:.2} reference_owned_bytes={reference_bytes} view_additional_bytes={view_bytes}",
        reference_ns[SAMPLES / 2],
        view_ns[SAMPLES / 2],
        reference_ns[SAMPLES / 2] / view_ns[SAMPLES / 2]
    );
}

#[test]
#[ignore = "manual release-mode AR-0016 source-shape and concurrency matrix"]
fn measures_whitespace_representation_matrix() {
    let strip_program = compile_measurement_program(true);
    let preserve_program = compile_measurement_program(false);
    let workloads = [
        measurement_source("small-heavy", &item_source(16, true), 4_000),
        measurement_source("medium-heavy", &item_source(500, true), 1_000),
        measurement_source("medium-light", &item_source(500, false), 1_000),
        measurement_source("large-heavy", &item_source(2_000, true), 250),
        measurement_source("deep-heavy", &deep_source(48), 1_000),
    ];

    for workload in &workloads {
        let reference = run_transform(
            &strip_program,
            &workload.source,
            WhitespaceRepresentation::CompleteReference,
            "reference-parity",
        );
        let view = run_transform(
            &strip_program,
            &workload.source,
            WhitespaceRepresentation::VisibilityView,
            "view-parity",
        );
        assert_eq!(reference, view, "{} semantic parity", workload.name);

        let construction_iterations = workload.iterations.max(250);
        let reference_construction_ns = median_batched_ns(construction_iterations, || {
            let mut control = InvocationControl::unbounded();
            black_box(
                workload
                    .source
                    .derive_stripping_all_element_whitespace(&mut control)
                    .expect("reference construction should succeed"),
            );
        });
        let view_construction_ns = median_batched_ns(construction_iterations, || {
            let mut control = InvocationControl::unbounded();
            black_box(
                workload
                    .source
                    .view_stripping_all_element_whitespace(&mut control)
                    .expect("view construction should succeed"),
            );
        });
        let preserve_ns = median_batched_ns(workload.iterations, || {
            black_box(run_transform(
                &preserve_program,
                &workload.source,
                WhitespaceRepresentation::VisibilityView,
                "preserve-warm",
            ));
        });
        let reference_ns = median_batched_ns(workload.iterations, || {
            black_box(run_transform(
                &strip_program,
                &workload.source,
                WhitespaceRepresentation::CompleteReference,
                "reference-warm",
            ));
        });
        let view_ns = median_batched_ns(workload.iterations, || {
            black_box(run_transform(
                &strip_program,
                &workload.source,
                WhitespaceRepresentation::VisibilityView,
                "view-warm",
            ));
        });
        let (view_p50_ns, view_p95_ns, view_p99_ns) = latency_quantiles_ns(|| {
            black_box(run_transform(
                &strip_program,
                &workload.source,
                WhitespaceRepresentation::VisibilityView,
                "view-latency",
            ));
        });
        let concurrency_iterations = (workload.iterations / 4).max(100);
        let reference_concurrent = concurrent_throughput(
            &strip_program,
            &workload.source,
            WhitespaceRepresentation::CompleteReference,
            concurrency_iterations,
        );
        let view_concurrent = concurrent_throughput(
            &strip_program,
            &workload.source,
            WhitespaceRepresentation::VisibilityView,
            concurrency_iterations,
        );
        println!(
            "workload={} nodes={} iterations={} reference_construct_ns={reference_construction_ns:.1} view_construct_ns={view_construction_ns:.1} construct_speedup={:.2} preserve_total_ns={preserve_ns:.1} reference_total_ns={reference_ns:.1} view_total_ns={view_ns:.1} total_speedup={:.2} view_over_preserve={:.2} view_p50_ns={view_p50_ns:.1} view_p95_ns={view_p95_ns:.1} view_p99_ns={view_p99_ns:.1} reference_concurrent_per_sec={reference_concurrent:.1} view_concurrent_per_sec={view_concurrent:.1} concurrent_speedup={:.2}",
            workload.name,
            workload.source.node_count(),
            workload.iterations,
            reference_construction_ns / view_construction_ns,
            reference_ns / view_ns,
            view_ns / preserve_ns,
            view_concurrent / reference_concurrent,
        );
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual allocator-requested AR-0016 peak and retained memory probe"]
fn measures_whitespace_representation_allocations() {
    let source = measurement_source("allocation-heavy", &item_source(2_000, true), 1).source;
    let strip_program = compile_measurement_program(true);
    let preserve_program = compile_measurement_program(false);

    let mut reference = None;
    let reference_construction = allocation_counter::measure(|| {
        let mut control = InvocationControl::unbounded();
        reference = Some(
            source
                .derive_stripping_all_element_whitespace(&mut control)
                .expect("reference construction should succeed"),
        );
    });
    let mut view = None;
    let view_construction = allocation_counter::measure(|| {
        let mut control = InvocationControl::unbounded();
        view = Some(
            source
                .view_stripping_all_element_whitespace(&mut control)
                .expect("view construction should succeed"),
        );
    });
    black_box(&reference);
    black_box(&view);
    assert!(reference_construction.bytes_current > view_construction.bytes_current);
    assert!(reference_construction.bytes_max > view_construction.bytes_max);

    let preserve_total = allocation_counter::measure(|| {
        black_box(run_transform(
            &preserve_program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            "preserve-allocation",
        ));
    });
    let reference_total = allocation_counter::measure(|| {
        black_box(run_transform(
            &strip_program,
            &source,
            WhitespaceRepresentation::CompleteReference,
            "reference-allocation",
        ));
    });
    let view_total = allocation_counter::measure(|| {
        black_box(run_transform(
            &strip_program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            "view-allocation",
        ));
    });
    assert!(reference_total.bytes_max > view_total.bytes_max);
    println!(
        "workload=allocation-heavy nodes={} reference_construction={reference_construction:?} view_construction={view_construction:?} preserve_total={preserve_total:?} reference_total={reference_total:?} view_total={view_total:?}",
        source.node_count(),
    );
}
