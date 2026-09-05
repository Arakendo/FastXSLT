use std::{fmt::Write as _, hint::black_box, time::Instant};

use crate::execution_control_experiment::WorkDomain;
use crate::xpath::decimal_sum_for_experiment::{
    DecimalSumForExpression, evaluate as evaluate_decimal_sum,
};
use crate::xslt::golden_semantics_experiment::{Instruction, ValueExpression};

use super::{
    CancellationToken, ExperimentalEngine, InvocationControl, WorkbenchLimits, execute_program,
    serialize_xml, serialize_xml_complete_namespace_reference, work_limits,
};

#[test]
#[ignore = "manual release-mode workbench transform phase probe"]
fn measure_workbench_transform_phases() {
    for (items, iterations) in [(5_usize, 10_000_usize), (50, 4_000), (500, 1_000)] {
        let engine = build_engine(items);
        let request = format!("workbench-transform-phase-{items}");
        let expected = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{items}.00</out>");
        let expected_decimal = format!("{items}.00");
        assert_eq!(
            engine.transform(&request).expect("warm transform"),
            expected
        );
        let document = engine
            .prepared
            .get(&engine.source_id)
            .expect("prepared source");
        let order = document
            .children(document.document_node())
            .iter()
            .copied()
            .find(|node| {
                document
                    .name(*node)
                    .is_some_and(|name| name.namespace.is_none() && name.local == "order")
            })
            .expect("order document element");
        let decimal = find_decimal_expression(&engine.program.matched_templates[0].template.body)
            .expect("compiled decimal sum expression");

        for sample in 1..=5 {
            let mut source_lookup = 0.0;
            let mut control_construction = 0.0;
            let mut semantic_execution = 0.0;
            let mut decimal_evaluation = 0.0;
            let mut serialization = 0.0;

            for _ in 0..iterations {
                let started = Instant::now();
                let document = engine
                    .prepared
                    .get(&engine.source_id)
                    .expect("prepared source");
                source_lookup += started.elapsed().as_secs_f64();

                let started = Instant::now();
                let mut control =
                    InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
                control_construction += started.elapsed().as_secs_f64();

                let started = Instant::now();
                let semantic = execute_program(&engine.program, &document, &request, &mut control)
                    .expect("semantic execution");
                semantic_execution += started.elapsed().as_secs_f64();

                let mut decimal_control =
                    InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
                let started = Instant::now();
                let decimal_value =
                    evaluate_decimal_sum(decimal, &document, order, &mut decimal_control)
                        .expect("decimal evaluation");
                decimal_evaluation += started.elapsed().as_secs_f64();
                assert_eq!(decimal_value, expected_decimal);

                let started = Instant::now();
                let serialized = serialize_xml(
                    &semantic,
                    &engine.program.output,
                    &request,
                    engine.limits.max_result_bytes,
                    &mut control,
                )
                .expect("serialization");
                serialization += started.elapsed().as_secs_f64();
                assert_eq!(serialized, expected);
            }

            let divisor = f64::from(
                u32::try_from(iterations).expect("measurement iteration count should fit u32"),
            );
            println!(
                "sample={sample} items={items} iterations={iterations} source_lookup_us={:.6} control_construction_us={:.6} semantic_execution_us={:.6} decimal_evaluation_us={:.6} serialization_us={:.6}",
                source_lookup * 1_000_000.0 / divisor,
                control_construction * 1_000_000.0 / divisor,
                semantic_execution * 1_000_000.0 / divisor,
                decimal_evaluation * 1_000_000.0 / divisor,
                serialization * 1_000_000.0 / divisor,
            );
        }
    }
}

#[test]
#[ignore = "manual release-mode for-004 production-shaped work-control replay"]
fn measure_for004_work_control_mix() {
    const SAMPLES: usize = 5;
    for (items, iterations) in [
        (5_usize, 100_000_usize),
        (50, 20_000),
        (500, 2_000),
        (5_000, 200),
    ] {
        let mut baseline_microseconds = Vec::with_capacity(SAMPLES);
        let mut charged_microseconds = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let started = Instant::now();
            for _ in 0..iterations {
                replay_for004_charge_shape(items, |domain| {
                    black_box(domain);
                });
            }
            baseline_microseconds.push(
                started.elapsed().as_secs_f64() * 1_000_000.0
                    / f64::from(u32::try_from(iterations).expect("iterations fit u32")),
            );

            let mut control = InvocationControl::unbounded();
            let started = Instant::now();
            for _ in 0..iterations {
                replay_for004_charge_shape(items, |domain| {
                    black_box(control.charge(domain, 1)).expect("unbounded charge");
                });
            }
            charged_microseconds.push(
                started.elapsed().as_secs_f64() * 1_000_000.0
                    / f64::from(u32::try_from(iterations).expect("iterations fit u32")),
            );

            assert_eq!(
                control.consumed(WorkDomain::XPathNodeVisit),
                iterations * items * 4
            );
            assert_eq!(
                control.consumed(WorkDomain::XdmStringValueNode),
                iterations * items * 2
            );
            assert_eq!(
                control.consumed(WorkDomain::XPathOperation),
                iterations * (items * 2 + 1)
            );
        }
        baseline_microseconds.sort_by(f64::total_cmp);
        charged_microseconds.sort_by(f64::total_cmp);
        let baseline = baseline_microseconds[SAMPLES / 2];
        let charged = charged_microseconds[SAMPLES / 2];
        println!(
            "items={items} iterations={iterations} charges={} baseline_us={baseline:.6} charged_us={charged:.6} delta_us={:.6}",
            items * 8 + 1,
            charged - baseline
        );
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual release-mode result-heavy execution and serialization attribution"]
fn measure_result_heavy_transform_phases() {
    for (items, iterations) in [(100_usize, 1_000_usize), (1_000, 200), (5_000, 40)] {
        let engine = build_result_heavy_engine(items);
        measure_engine_phases(&engine, &format!("result-heavy-{items}"), iterations);
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual release-mode text-heavy execution and serialization attribution"]
fn measure_text_heavy_transform_phases() {
    for (bytes, iterations) in [(4_096_usize, 2_000_usize), (65_536, 200), (524_288, 20)] {
        let engine = build_text_heavy_engine(bytes);
        measure_engine_phases(&engine, &format!("text-heavy-{bytes}"), iterations);
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual release-mode namespace-heavy execution and serialization attribution"]
fn measure_namespace_heavy_transform_phases() {
    for (depth, iterations) in [(8_usize, 2_000_usize), (24, 500), (48, 100)] {
        let engine = build_namespace_heavy_engine(depth, 8);
        measure_engine_phases(
            &engine,
            &format!("namespace-heavy-depth-{depth}"),
            iterations,
        );
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual release-mode scoped-stack versus complete-clone serializer comparison"]
fn compare_namespace_scope_serializers() {
    compare_namespace_serializers(&build_engine(500), "ordinary-for-004-500", 5_000);
    for (depth, iterations) in [(8_usize, 2_000_usize), (24, 500), (48, 100)] {
        compare_namespace_serializers(
            &build_namespace_heavy_engine(depth, 8),
            &format!("namespace-heavy-depth-{depth}"),
            iterations,
        );
    }
}

#[cfg(feature = "allocation-observation")]
fn compare_namespace_serializers(engine: &ExperimentalEngine, request: &str, iterations: usize) {
    let document = engine
        .prepared
        .get(&engine.source_id)
        .expect("prepared comparison source");
    let mut execution_control =
        InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
    let semantic = execute_program(&engine.program, &document, request, &mut execution_control)
        .expect("comparison semantic execution");

    let mut scoped_value = None;
    let mut scoped_control =
        InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
    let scoped_allocations = allocation_counter::measure(|| {
        scoped_value = Some(
            serialize_xml(
                &semantic,
                &engine.program.output,
                request,
                engine.limits.max_result_bytes,
                &mut scoped_control,
            )
            .expect("scoped namespace serialization"),
        );
    });
    let mut complete_value = None;
    let mut complete_control =
        InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
    let complete_allocations = allocation_counter::measure(|| {
        complete_value = Some(
            serialize_xml_complete_namespace_reference(
                &semantic,
                &engine.program.output,
                request,
                engine.limits.max_result_bytes,
                &mut complete_control,
            )
            .expect("complete namespace serialization"),
        );
    });
    assert_eq!(scoped_value, complete_value);
    assert_eq!(
        scoped_control.consumed(WorkDomain::SerializedByte),
        complete_control.consumed(WorkDomain::SerializedByte)
    );

    let mut scoped_samples = Vec::with_capacity(5);
    let mut complete_samples = Vec::with_capacity(5);
    for _ in 0..5 {
        let mut scoped_seconds = 0.0;
        let mut complete_seconds = 0.0;
        for _ in 0..iterations {
            let mut complete_control =
                InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
            let started = Instant::now();
            let complete = serialize_xml_complete_namespace_reference(
                &semantic,
                &engine.program.output,
                request,
                engine.limits.max_result_bytes,
                &mut complete_control,
            )
            .expect("timed complete namespace serialization");
            complete_seconds += started.elapsed().as_secs_f64();

            let mut scoped_control =
                InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
            let started = Instant::now();
            let scoped = serialize_xml(
                &semantic,
                &engine.program.output,
                request,
                engine.limits.max_result_bytes,
                &mut scoped_control,
            )
            .expect("timed scoped namespace serialization");
            scoped_seconds += started.elapsed().as_secs_f64();
            assert_eq!(scoped, complete);
        }
        let divisor = f64::from(u32::try_from(iterations).expect("iterations fit u32"));
        scoped_samples.push(scoped_seconds * 1_000_000.0 / divisor);
        complete_samples.push(complete_seconds * 1_000_000.0 / divisor);
    }
    scoped_samples.sort_by(f64::total_cmp);
    complete_samples.sort_by(f64::total_cmp);
    eprintln!(
        "request={request} iterations={iterations} scoped_median_us={:.6} complete_median_us={:.6} scoped_allocations={scoped_allocations:?} complete_allocations={complete_allocations:?}",
        scoped_samples[2], complete_samples[2]
    );
}

#[cfg(feature = "allocation-observation")]
fn measure_engine_phases(engine: &ExperimentalEngine, request: &str, iterations: usize) {
    let expected = engine.transform(request).expect("warm measured transform");
    let document = engine
        .prepared
        .get(&engine.source_id)
        .expect("prepared measured source");

    let mut retained_semantic = None;
    let mut allocation_control =
        InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
    let execution_allocations = allocation_counter::measure(|| {
        retained_semantic = Some(
            execute_program(&engine.program, &document, request, &mut allocation_control)
                .expect("allocation-observed semantic execution"),
        );
    });
    let semantic = retained_semantic
        .as_ref()
        .expect("allocation-observed result retained");
    let mut serialization_control =
        InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
    let serialization_allocations = allocation_counter::measure(|| {
        let serialized = serialize_xml(
            semantic,
            &engine.program.output,
            request,
            engine.limits.max_result_bytes,
            &mut serialization_control,
        )
        .expect("allocation-observed serialization");
        assert_eq!(serialized, expected);
        black_box(serialized);
    });

    let mut execution_samples = Vec::with_capacity(5);
    let mut serialization_samples = Vec::with_capacity(5);
    for _ in 0..5 {
        let mut execution_seconds = 0.0;
        let mut serialization_seconds = 0.0;
        for _ in 0..iterations {
            let mut control =
                InvocationControl::new(CancellationToken::new(), work_limits(engine.limits));
            let started = Instant::now();
            let semantic = execute_program(&engine.program, &document, request, &mut control)
                .expect("measured semantic execution");
            execution_seconds += started.elapsed().as_secs_f64();

            let started = Instant::now();
            let serialized = serialize_xml(
                &semantic,
                &engine.program.output,
                request,
                engine.limits.max_result_bytes,
                &mut control,
            )
            .expect("measured serialization");
            serialization_seconds += started.elapsed().as_secs_f64();
            assert_eq!(serialized, expected);
        }
        let divisor = f64::from(u32::try_from(iterations).expect("iterations fit u32"));
        execution_samples.push(execution_seconds * 1_000_000.0 / divisor);
        serialization_samples.push(serialization_seconds * 1_000_000.0 / divisor);
    }
    execution_samples.sort_by(f64::total_cmp);
    serialization_samples.sort_by(f64::total_cmp);
    println!(
        "request={request} iterations={iterations} result_bytes={} execution_median_us={:.6} serialization_median_us={:.6} execution_allocations={execution_allocations:?} serialization_allocations={serialization_allocations:?}",
        expected.len(),
        execution_samples[2],
        serialization_samples[2],
    );
}

fn replay_for004_charge_shape(items: usize, mut charge: impl FnMut(WorkDomain)) {
    for _ in 0..items {
        charge(WorkDomain::XPathNodeVisit);
    }
    for _ in 0..items {
        charge(WorkDomain::XPathNodeVisit);
        charge(WorkDomain::XPathNodeVisit);
        charge(WorkDomain::XPathNodeVisit);
        charge(WorkDomain::XdmStringValueNode);
        charge(WorkDomain::XdmStringValueNode);
        charge(WorkDomain::XPathOperation);
        charge(WorkDomain::XPathOperation);
    }
    charge(WorkDomain::XPathOperation);
}

fn find_decimal_expression(instructions: &[Instruction]) -> Option<&DecimalSumForExpression> {
    instructions
        .iter()
        .find_map(|instruction| match instruction {
            Instruction::ValueOf {
                select: ValueExpression::DecimalSumFor(expression),
                ..
            } => Some(expression.as_ref()),
            Instruction::LiteralElement { body, .. } => find_decimal_expression(body),
            _ => None,
        })
}

fn build_engine(items: usize) -> ExperimentalEngine {
    let mut source = String::from("<?xml version=\"1.0\"?><order>");
    for _ in 0..items {
        source.push_str("<order-item price=\"1.00\" qty=\"1\"/>");
    }
    source.push_str("</order>");
    ExperimentalEngine::new(
        format!("urn:fastxslt:workbench-transform-phase:{items}:source"),
        source.into_bytes(),
        format!("urn:fastxslt:workbench-transform-phase:{items}:stylesheet"),
        include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
        WorkbenchLimits::default(),
    )
    .expect("build measured engine")
}

#[cfg(feature = "allocation-observation")]
fn build_result_heavy_engine(items: usize) -> ExperimentalEngine {
    let stylesheet = format!(
        r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="1 to {items}"><item code="fixed">payload</item></xsl:for-each></out></xsl:template></xsl:stylesheet>"#
    );
    ExperimentalEngine::new(
        format!("urn:fastxslt:result-heavy:{items}:source"),
        b"<root/>".to_vec(),
        format!("urn:fastxslt:result-heavy:{items}:stylesheet"),
        stylesheet.into_bytes(),
        WorkbenchLimits::default(),
    )
    .expect("build result-heavy measured engine")
}

#[cfg(feature = "allocation-observation")]
fn build_text_heavy_engine(bytes: usize) -> ExperimentalEngine {
    let text = "a".repeat(bytes);
    let stylesheet = format!(
        r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out>{text}</out></xsl:template></xsl:stylesheet>"#
    );
    let limits = WorkbenchLimits {
        max_result_bytes: bytes + 1_024,
        max_resource_bytes: bytes + 4_096,
        ..WorkbenchLimits::default()
    };
    ExperimentalEngine::new(
        format!("urn:fastxslt:text-heavy:{bytes}:source"),
        b"<root/>".to_vec(),
        format!("urn:fastxslt:text-heavy:{bytes}:stylesheet"),
        stylesheet.into_bytes(),
        limits,
    )
    .expect("build text-heavy measured engine")
}

#[cfg(feature = "allocation-observation")]
fn build_namespace_heavy_engine(depth: usize, bindings_per_element: usize) -> ExperimentalEngine {
    let mut stylesheet = String::from(
        r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/">"#,
    );
    for level in 0..depth {
        write!(&mut stylesheet, "<p{level}_0:e{level}").expect("write namespace fixture");
        for binding in 0..bindings_per_element {
            write!(
                &mut stylesheet,
                " xmlns:p{level}_{binding}=\"urn:fastxslt:namespace-heavy:{level}:{binding}\""
            )
            .expect("write namespace binding");
        }
        write!(&mut stylesheet, " p{level}_0:a=\"{level}\">").expect("write namespace attribute");
    }
    stylesheet.push_str("payload");
    for level in (0..depth).rev() {
        write!(&mut stylesheet, "</p{level}_0:e{level}>").expect("write namespace fixture close");
    }
    stylesheet.push_str("</xsl:template></xsl:stylesheet>");
    let limits = WorkbenchLimits {
        max_resource_bytes: stylesheet.len() + 1_024,
        max_xml_depth: depth + 8,
        ..WorkbenchLimits::default()
    };
    ExperimentalEngine::new(
        format!("urn:fastxslt:namespace-heavy:{depth}:source"),
        b"<root/>".to_vec(),
        format!("urn:fastxslt:namespace-heavy:{depth}:stylesheet"),
        stylesheet.into_bytes(),
        limits,
    )
    .expect("build namespace-heavy measured engine")
}
