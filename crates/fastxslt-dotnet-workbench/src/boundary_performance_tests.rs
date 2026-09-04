use std::{hint::black_box, sync::Arc, time::Instant};

use fastxslt::workbench::{ExperimentalEngine, WorkbenchLimits};

use super::{
    MAX_IDENTITY_BYTES, State, configured_test_state, copy_input, decode_identity,
    insert_transform_outcome,
};

#[test]
#[ignore = "manual release-mode native transform-export component probe"]
fn measure_native_transform_export_components() {
    for (items, iterations) in [(5, 10_000), (50, 4_000), (500, 1_000)] {
        let state = configured_test_state();
        let creation = state.insert_created_engine(build_engine(items));
        let engine_handle = state
            .take_engine_outcome(creation)
            .expect("creation outcome owns the measured engine");
        let engine = retained_engine(&state, engine_handle);
        let request = format!("native-export-component-{items}");
        let expected = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{items}.00</out>");
        assert_eq!(
            engine.transform(&request).expect("warm transform"),
            expected
        );

        let mut request_decode = 0.0;
        let mut engine_lookup = 0.0;
        let mut transform = 0.0;
        let mut outcome_insert = 0.0;
        let mut outcome_release = 0.0;

        for _ in 0..iterations {
            let started = Instant::now();
            let decoded = decode_identity(
                copy_input(request.as_ptr(), request.len(), MAX_IDENTITY_BYTES)
                    .expect("copy request identity"),
                "request identity",
            )
            .expect("decode request identity");
            request_decode += started.elapsed().as_secs_f64();

            let started = Instant::now();
            let acquired = retained_engine(&state, engine_handle);
            engine_lookup += started.elapsed().as_secs_f64();

            let started = Instant::now();
            let result = acquired.transform(&decoded).expect("measured transform");
            transform += started.elapsed().as_secs_f64();
            assert_eq!(result, expected);

            let started = Instant::now();
            let outcome = insert_transform_outcome(&state, Ok(result));
            outcome_insert += started.elapsed().as_secs_f64();

            let started = Instant::now();
            assert!(state.release_outcome(outcome));
            outcome_release += started.elapsed().as_secs_f64();
            black_box(acquired);
        }

        let divisor = f64::from(iterations);
        println!(
            "items={items} iterations={iterations} request_decode_us={:.6} engine_lookup_us={:.6} transform_us={:.6} outcome_insert_us={:.6} outcome_release_us={:.6}",
            request_decode * 1_000_000.0 / divisor,
            engine_lookup * 1_000_000.0 / divisor,
            transform * 1_000_000.0 / divisor,
            outcome_insert * 1_000_000.0 / divisor,
            outcome_release * 1_000_000.0 / divisor,
        );
        assert!(state.release_engine(engine_handle));
    }
}

fn retained_engine(state: &State, handle: u64) -> Arc<ExperimentalEngine> {
    state
        .engines()
        .expect("engine registry")
        .get(&handle)
        .map(|entry| Arc::clone(&entry.engine))
        .expect("measured engine handle")
}

fn build_engine(items: usize) -> ExperimentalEngine {
    let mut source = String::from("<?xml version=\"1.0\"?><order>");
    for _ in 0..items {
        source.push_str("<order-item price=\"1.00\" qty=\"1\"/>");
    }
    source.push_str("</order>");
    ExperimentalEngine::new(
        format!("urn:fastxslt:native-export-component:{items}:source"),
        source.into_bytes(),
        format!("urn:fastxslt:native-export-component:{items}:stylesheet"),
        include_bytes!("../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
        WorkbenchLimits::default(),
    )
    .expect("build measured engine")
}
