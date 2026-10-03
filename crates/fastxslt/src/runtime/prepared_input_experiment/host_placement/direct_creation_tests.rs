//! Matched full-engine creation control, not a compile-once preparation lane.

use std::{hint::black_box, time::Instant};

use crate::runtime::workbench_experiment::{ExperimentalEngine, WorkbenchLimits};

struct Fixture {
    name: String,
    source: Vec<u8>,
    expected: String,
}

fn fixtures() -> Vec<Fixture> {
    let mut fixtures = vec![Fixture {
        name: "pinned-for-004".into(),
        source: super::SOURCE.to_vec(),
        expected: super::EXPECTED.into(),
    }];
    for items in [5, 50, 500] {
        fixtures.push(Fixture {
            name: format!("items-{items}"),
            source: format!(
                "<?xml version=\"1.0\"?><order>{}</order>",
                "<order-item price=\"1.00\" qty=\"1\"/>".repeat(items)
            )
            .into_bytes(),
            expected: format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{items}.00</out>"),
        });
    }
    fixtures
}

fn sample(fixture: &Fixture) -> [u128; 4] {
    let total = Instant::now();
    let creation = Instant::now();
    let engine = ExperimentalEngine::new(
        format!("urn:ar0027:creation:{}:source", fixture.name),
        fixture.source.clone(),
        "urn:ar0027:creation:stylesheet",
        super::STYLE.to_vec(),
        WorkbenchLimits::default(),
    )
    .unwrap();
    let creation = creation.elapsed().as_nanos();
    let transform = Instant::now();
    let result = engine.transform("urn:ar0027:creation:request").unwrap();
    assert_eq!(result, fixture.expected);
    black_box(&result);
    let transform = transform.elapsed().as_nanos();
    // Like the adapter lanes, result ownership stays with the caller while the
    // engine is disposed. Result release is outside the lifecycle clock.
    let disposal = Instant::now();
    drop(engine);
    let disposal = disposal.elapsed().as_nanos();
    [creation, transform, disposal, total.elapsed().as_nanos()]
}

#[test]
fn full_creation_preserves_matched_outputs_and_default_dtd_denial() {
    for fixture in fixtures() {
        let phases = sample(&fixture);
        assert!(phases[3] >= phases[0] + phases[1] + phases[2]);
    }
    for source in ["<order>", "<!DOCTYPE order><order/>"] {
        let failure = ExperimentalEngine::new(
            "urn:ar0027:creation:invalid-source",
            source.as_bytes().to_vec(),
            "urn:ar0027:creation:stylesheet",
            super::STYLE.to_vec(),
            WorkbenchLimits::default(),
        )
        .err()
        .expect("invalid source must not create an engine");
        assert!(!failure.code.is_empty());
        assert!(!failure.category.is_empty());
        assert!(failure.request_id.is_none());
        println!("ar0027-creation failure={failure:?}");
    }
}

#[test]
#[ignore = "manual release matched creation measurement; phase clocks add overhead"]
fn measure_matched_full_creation() {
    let order =
        std::env::var("AR0027_CREATION_ORDER").map_or(0, |value| value.parse::<usize>().unwrap());
    assert!(order <= 2);
    let fixtures = fixtures();
    for group in 0..fixtures.len() {
        let fixture = &fixtures[(group + order) % fixtures.len()];
        for _ in 0..8 {
            sample(fixture);
        }
        let samples: Vec<_> = (0..32).map(|_| sample(fixture)).collect();
        for (index, phase) in [
            "creation",
            "first-transform-validation",
            "disposal",
            "lifecycle",
        ]
        .into_iter()
        .enumerate()
        {
            let mut values: Vec<_> = samples.iter().map(|sample| sample[index]).collect();
            values.sort_unstable();
            println!(
                "ar0027-creation order={order} fixture={} source_bytes={} phase={phase} samples=32 median_ns={} p95_ns={} min_ns={} max_ns={}",
                fixture.name,
                fixture.source.len(),
                u128::midpoint(values[15], values[16]),
                values[30],
                values[0],
                values[31],
            );
        }
    }
}
