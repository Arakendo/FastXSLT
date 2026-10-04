//! Complete new-engine timing; existing growth generation is an untimed lease.

use super::{CAPACITIES, PreparationCapacity, STYLE, WorkbenchLimits, create, engine, source};
use std::{hint::black_box, time::Instant};

fn sample(
    items: usize,
    capacity: PreparationCapacity,
    uses: usize,
    overlap: bool,
    expected: &str,
) -> [u128; 4] {
    let old = overlap.then(|| engine(items, 8, PreparationCapacity::Growth));
    let total = Instant::now();
    let phase = Instant::now();
    let new = create(
        items,
        9,
        capacity,
        WorkbenchLimits::default(),
        expected.as_bytes().to_vec(),
        STYLE.to_vec(),
    )
    .unwrap();
    let creation = phase.elapsed().as_nanos();
    let mut transform = 0;
    let mut release = 0;
    for _ in 0..uses {
        let phase = Instant::now();
        let output = new.transform_bytes("engine-timing").unwrap();
        transform += phase.elapsed().as_nanos();
        assert_eq!(output, expected.as_bytes());
        black_box(&output);
        let phase = Instant::now();
        drop(output);
        release += phase.elapsed().as_nanos();
    }
    let phase = Instant::now();
    drop(new);
    release += phase.elapsed().as_nanos();
    let total = total.elapsed().as_nanos();
    if let Some(old) = old {
        assert_eq!(
            old.transform_bytes("old-after-new-retirement").unwrap(),
            source(items, 8).as_bytes()
        );
    }
    [creation, transform, release, total]
}

#[test]
fn complete_engine_timing_scope_preserves_exact_output_and_old_generation() {
    for items in [5, 50, 500] {
        let expected = source(items, 9);
        for uses in [1, 8] {
            for overlap in [false, true] {
                for capacity in CAPACITIES {
                    sample(items, capacity, uses, overlap, &expected);
                }
            }
        }
    }
}

#[test]
#[ignore = "manual release complete-engine timing; short windows are not publication eligible"]
fn measure_complete_engine_capacity_timing() {
    let order = std::env::var("AR0027_ENGINE_ORDER").map_or(0, |v| v.parse::<usize>().unwrap());
    assert!(order < 3);
    for item_index in 0..3 {
        let items = [5, 50, 500][(item_index + order) % 3];
        let expected = source(items, 9);
        for uses in if order == 1 { [8, 1] } else { [1, 8] } {
            for overlap in [false, true] {
                for offset in 0..3 {
                    let capacity = CAPACITIES[(offset + order) % 3];
                    for _ in 0..8 {
                        sample(items, capacity, uses, overlap, &expected);
                    }
                    let samples: Vec<_> = (0..32)
                        .map(|_| sample(items, capacity, uses, overlap, &expected))
                        .collect();
                    for phase in 0..4 {
                        let mut values: Vec<_> =
                            samples.iter().map(|sample| sample[phase]).collect();
                        values.sort_unstable();
                        println!(
                            "ar0027-engine-timing order={order} items={items} uses={uses} overlap={overlap} capacity={capacity:?} phase={phase} median_ns={} p95_ns={} publication_eligible=false",
                            u128::midpoint(values[15], values[16]),
                            values[30]
                        );
                    }
                }
            }
        }
    }
}
