use std::time::Instant;

use crate::xpath::decimal_sum_for_experiment::{
    DecimalSumForExpression, evaluate as evaluate_decimal_sum,
};
use crate::xslt::golden_semantics_experiment::{Instruction, ValueExpression};

use super::{
    CancellationToken, ExperimentalEngine, InvocationControl, WorkbenchLimits, execute_program,
    serialize_xml, work_limits,
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
