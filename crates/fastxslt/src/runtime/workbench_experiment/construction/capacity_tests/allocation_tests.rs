//! Fresh non-nested complete-engine prefix scopes; requested bytes, not RSS.

use super::{
    CAPACITIES, ExperimentalEngine, PreparationCapacity, STYLE, WorkbenchCancellation,
    WorkbenchLimits, create, source,
};
use allocation_counter::{AllocationInfo, measure};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    OldEngine,
    Overlap,
    DelayedResults,
    OnlyResults,
    Released,
    FailedReplacement,
    CancelledInvocation,
}

const STAGES: [Stage; 7] = [
    Stage::OldEngine,
    Stage::Overlap,
    Stage::DelayedResults,
    Stage::OnlyResults,
    Stage::Released,
    Stage::FailedReplacement,
    Stage::CancelledInvocation,
];

fn build(
    items: usize,
    job: usize,
    capacity: PreparationCapacity,
    bytes: &[u8],
) -> Arc<ExperimentalEngine> {
    Arc::new(
        create(
            items,
            job,
            capacity,
            WorkbenchLimits::default(),
            bytes.to_vec(),
            STYLE.to_vec(),
        )
        .unwrap(),
    )
}

fn scope(
    items: usize,
    capacity: PreparationCapacity,
    stage: Stage,
    sources: &[String; 2],
) -> (AllocationInfo, usize) {
    let mut retained = None;
    let mut known_engine_bytes = 0;
    let info = measure(|| {
        let old = build(items, 8, capacity, sources[0].as_bytes());
        if stage == Stage::FailedReplacement {
            let failure = create(
                items,
                9,
                capacity,
                WorkbenchLimits::default(),
                b"<root>".to_vec(),
                STYLE.to_vec(),
            )
            .err()
            .unwrap();
            assert_eq!(failure.category, "invalid");
            drop(failure);
            assert_eq!(
                old.transform_bytes("failed-replacement-recovery").unwrap(),
                sources[0].as_bytes()
            );
            return;
        }
        if stage == Stage::CancelledInvocation {
            let token = WorkbenchCancellation::new();
            token.cancel();
            let failure = old
                .transform_with_cancellation("allocation-cancel", token)
                .unwrap_err();
            assert_eq!(
                (failure.code.as_str(), failure.category.as_str()),
                ("FXCT0001", "cancelled")
            );
            drop(failure);
            assert_eq!(
                old.transform_bytes("cancel-recovery").unwrap(),
                sources[0].as_bytes()
            );
            return;
        }
        let mut engines = vec![old];
        if stage != Stage::OldEngine {
            engines.push(build(items, 9, capacity, sources[1].as_bytes()));
        }
        let mut outcomes = Vec::new();
        if matches!(
            stage,
            Stage::DelayedResults | Stage::OnlyResults | Stage::Released
        ) {
            for (index, engine) in engines.iter().enumerate() {
                let output = engine.transform_bytes("engine-allocation").unwrap();
                assert_eq!(output, sources[index].as_bytes());
                outcomes.push(output);
            }
        }
        if stage == Stage::OnlyResults {
            drop(engines);
            engines = Vec::new();
            for (index, output) in outcomes.iter().enumerate() {
                assert_eq!(output, sources[index].as_bytes());
            }
        }
        if stage != Stage::Released {
            known_engine_bytes = engines
                .iter()
                .map(|engine| engine.retention_estimate().known_retained_capacity_bytes)
                .sum();
            retained = Some((engines, outcomes));
        }
    });
    if matches!(
        stage,
        Stage::Released | Stage::FailedReplacement | Stage::CancelledInvocation
    ) {
        assert_eq!((info.count_current, info.bytes_current), (0, 0));
    } else {
        assert!(info.bytes_current > 0);
        assert!(known_engine_bytes <= usize::try_from(info.bytes_current).unwrap());
    }
    drop(retained);
    (info, known_engine_bytes)
}

fn run(print: bool) {
    for items in [5, 50, 500] {
        // Caller fixture creation is outside the scope; engine-owned copying is inside.
        let sources = [source(items, 8), source(items, 9)];
        for capacity in CAPACITIES {
            for stage in STAGES {
                let (info, known) = scope(items, capacity, stage, &sources);
                if print {
                    println!(
                        "ar0027-engine-allocation items={items} capacity={capacity:?} stage={stage:?} known_engine_bytes={known} count_total={} count_current={} count_max={} bytes_total={} bytes_current={} bytes_max={}",
                        info.count_total,
                        info.count_current,
                        info.count_max,
                        info.bytes_total,
                        info.bytes_current,
                        info.bytes_max
                    );
                }
            }
        }
    }
}

#[test]
fn complete_engine_prefix_scopes_release_success_failure_and_cancellation() {
    run(false);
}

#[test]
#[ignore = "manual release complete-engine overlap allocation probe; not RSS"]
fn measure_complete_engine_overlap_allocations() {
    run(true);
}
