use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectionDisposition {
    Selected,
    ExcludedByProfile,
    EngineUnsupported,
    HarnessUnsupported,
    MetadataFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InventoryCase {
    identity: &'static str,
    selection: SelectionDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionDisposition {
    Passed,
    SemanticMismatch,
    EngineFailure,
    HarnessFailure,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AttemptObservation {
    case_identity: &'static str,
    attempt: u32,
    execution: ExecutionDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShardReport {
    run_identity: &'static str,
    shard: u16,
    shard_count: u16,
    observations: Vec<AttemptObservation>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SelectionTotals {
    discovered: usize,
    selected: usize,
    excluded_by_profile: usize,
    engine_unsupported: usize,
    harness_unsupported: usize,
    metadata_failures: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ExecutionTotals {
    selected: usize,
    passed: usize,
    semantic_mismatches: usize,
    engine_failures: usize,
    harness_failures: usize,
    incomplete: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MergedLedger {
    selection: SelectionTotals,
    execution: ExecutionTotals,
    final_outcomes: BTreeMap<&'static str, ExecutionDisposition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MergeFailure {
    DuplicateInventoryCase,
    InvalidShardCoordinates,
    MixedRunIdentity,
    MixedShardTopology,
    UnknownCase,
    ExecutionForUnselectedCase,
    ConflictingAttempt,
}

fn selection_totals(inventory: &[InventoryCase]) -> SelectionTotals {
    let mut totals = SelectionTotals {
        discovered: inventory.len(),
        ..SelectionTotals::default()
    };
    for case in inventory {
        match case.selection {
            SelectionDisposition::Selected => totals.selected += 1,
            SelectionDisposition::ExcludedByProfile => totals.excluded_by_profile += 1,
            SelectionDisposition::EngineUnsupported => totals.engine_unsupported += 1,
            SelectionDisposition::HarnessUnsupported => totals.harness_unsupported += 1,
            SelectionDisposition::MetadataFailure => totals.metadata_failures += 1,
        }
    }
    totals
}

fn merge_reports(
    inventory: &[InventoryCase],
    reports: &[ShardReport],
) -> Result<MergedLedger, MergeFailure> {
    let known: BTreeMap<_, _> = inventory
        .iter()
        .map(|case| (case.identity, case.selection))
        .collect();
    if known.len() != inventory.len() {
        return Err(MergeFailure::DuplicateInventoryCase);
    }
    let mut latest = BTreeMap::<&str, (u32, ExecutionDisposition)>::new();
    let expected_run = reports.first().map(|report| report.run_identity);
    let expected_shard_count = reports.first().map(|report| report.shard_count);

    for report in reports {
        if report.shard_count == 0 || report.shard >= report.shard_count {
            return Err(MergeFailure::InvalidShardCoordinates);
        }
        if Some(report.run_identity) != expected_run {
            return Err(MergeFailure::MixedRunIdentity);
        }
        if Some(report.shard_count) != expected_shard_count {
            return Err(MergeFailure::MixedShardTopology);
        }
        for observation in &report.observations {
            let selection = known
                .get(observation.case_identity)
                .ok_or(MergeFailure::UnknownCase)?;
            if *selection != SelectionDisposition::Selected {
                return Err(MergeFailure::ExecutionForUnselectedCase);
            }
            match latest.get(observation.case_identity) {
                Some((attempt, execution)) if *attempt == observation.attempt => {
                    if *execution != observation.execution {
                        return Err(MergeFailure::ConflictingAttempt);
                    }
                }
                Some((attempt, _)) if *attempt > observation.attempt => {}
                _ => {
                    latest.insert(
                        observation.case_identity,
                        (observation.attempt, observation.execution),
                    );
                }
            }
        }
    }

    let selection = selection_totals(inventory);
    let mut execution = ExecutionTotals {
        selected: selection.selected,
        ..ExecutionTotals::default()
    };
    let mut final_outcomes = BTreeMap::new();
    for case in inventory
        .iter()
        .filter(|case| case.selection == SelectionDisposition::Selected)
    {
        let outcome = latest
            .get(case.identity)
            .map_or(ExecutionDisposition::Incomplete, |(_, outcome)| *outcome);
        match outcome {
            ExecutionDisposition::Passed => execution.passed += 1,
            ExecutionDisposition::SemanticMismatch => execution.semantic_mismatches += 1,
            ExecutionDisposition::EngineFailure => execution.engine_failures += 1,
            ExecutionDisposition::HarnessFailure => execution.harness_failures += 1,
            ExecutionDisposition::Incomplete => execution.incomplete += 1,
        }
        final_outcomes.insert(case.identity, outcome);
    }

    Ok(MergedLedger {
        selection,
        execution,
        final_outcomes,
    })
}

fn filtered_inventory() -> [InventoryCase; 9] {
    [
        InventoryCase {
            identity: "selected-pass",
            selection: SelectionDisposition::Selected,
        },
        InventoryCase {
            identity: "selected-mismatch",
            selection: SelectionDisposition::Selected,
        },
        InventoryCase {
            identity: "selected-engine-failure",
            selection: SelectionDisposition::Selected,
        },
        InventoryCase {
            identity: "selected-harness-failure",
            selection: SelectionDisposition::Selected,
        },
        InventoryCase {
            identity: "selected-interrupted-then-pass",
            selection: SelectionDisposition::Selected,
        },
        InventoryCase {
            identity: "profile-excluded",
            selection: SelectionDisposition::ExcludedByProfile,
        },
        InventoryCase {
            identity: "engine-unsupported",
            selection: SelectionDisposition::EngineUnsupported,
        },
        InventoryCase {
            identity: "harness-unsupported",
            selection: SelectionDisposition::HarnessUnsupported,
        },
        InventoryCase {
            identity: "metadata-failure",
            selection: SelectionDisposition::MetadataFailure,
        },
    ]
}

fn shard_reports() -> [ShardReport; 2] {
    [
        ShardReport {
            run_identity: "private-ledger-run-0",
            shard: 0,
            shard_count: 2,
            observations: vec![
                AttemptObservation {
                    case_identity: "selected-pass",
                    attempt: 0,
                    execution: ExecutionDisposition::Passed,
                },
                AttemptObservation {
                    case_identity: "selected-engine-failure",
                    attempt: 0,
                    execution: ExecutionDisposition::EngineFailure,
                },
                AttemptObservation {
                    case_identity: "selected-interrupted-then-pass",
                    attempt: 0,
                    execution: ExecutionDisposition::Incomplete,
                },
            ],
        },
        ShardReport {
            run_identity: "private-ledger-run-0",
            shard: 1,
            shard_count: 2,
            observations: vec![
                AttemptObservation {
                    case_identity: "selected-mismatch",
                    attempt: 0,
                    execution: ExecutionDisposition::SemanticMismatch,
                },
                AttemptObservation {
                    case_identity: "selected-harness-failure",
                    attempt: 0,
                    execution: ExecutionDisposition::HarnessFailure,
                },
            ],
        },
    ]
}

#[test]
fn totals_conserve_across_filtering_shards_interruption_retry_and_merge_order() {
    let inventory = filtered_inventory();
    let shards = shard_reports();
    let interrupted = merge_reports(&inventory, &shards).expect("merge interrupted run");

    assert_eq!(
        interrupted.selection,
        SelectionTotals {
            discovered: 9,
            selected: 5,
            excluded_by_profile: 1,
            engine_unsupported: 1,
            harness_unsupported: 1,
            metadata_failures: 1,
        }
    );
    assert_eq!(
        interrupted.execution,
        ExecutionTotals {
            selected: 5,
            passed: 1,
            semantic_mismatches: 1,
            engine_failures: 1,
            harness_failures: 1,
            incomplete: 1,
        }
    );

    let retry = ShardReport {
        run_identity: "private-ledger-run-0",
        shard: 1,
        shard_count: 2,
        observations: vec![AttemptObservation {
            case_identity: "selected-interrupted-then-pass",
            attempt: 1,
            execution: ExecutionDisposition::Passed,
        }],
    };
    let forward = merge_reports(
        &inventory,
        &[shards[0].clone(), shards[1].clone(), retry.clone()],
    )
    .expect("merge forward completion order");
    let reverse = merge_reports(&inventory, &[retry, shards[1].clone(), shards[0].clone()])
        .expect("merge reverse completion order");

    assert_eq!(forward, reverse);
    assert_eq!(
        forward.execution,
        ExecutionTotals {
            selected: 5,
            passed: 2,
            semantic_mismatches: 1,
            engine_failures: 1,
            harness_failures: 1,
            incomplete: 0,
        }
    );
    assert_eq!(
        forward.selection.discovered,
        forward.selection.selected
            + forward.selection.excluded_by_profile
            + forward.selection.engine_unsupported
            + forward.selection.harness_unsupported
            + forward.selection.metadata_failures
    );
    assert_eq!(
        forward.execution.selected,
        forward.execution.passed
            + forward.execution.semantic_mismatches
            + forward.execution.engine_failures
            + forward.execution.harness_failures
            + forward.execution.incomplete
    );
}

#[test]
fn contradictory_or_unowned_observations_fail_the_merge() {
    let inventory = filtered_inventory();
    let invalid = ShardReport {
        run_identity: "private-ledger-run-0",
        shard: 0,
        shard_count: 1,
        observations: vec![AttemptObservation {
            case_identity: "profile-excluded",
            attempt: 0,
            execution: ExecutionDisposition::Passed,
        }],
    };
    assert_eq!(
        merge_reports(&inventory, &[invalid]),
        Err(MergeFailure::ExecutionForUnselectedCase)
    );

    let conflicting = ShardReport {
        run_identity: "private-ledger-run-0",
        shard: 0,
        shard_count: 1,
        observations: vec![
            AttemptObservation {
                case_identity: "selected-pass",
                attempt: 0,
                execution: ExecutionDisposition::Passed,
            },
            AttemptObservation {
                case_identity: "selected-pass",
                attempt: 0,
                execution: ExecutionDisposition::SemanticMismatch,
            },
        ],
    };
    assert_eq!(
        merge_reports(&inventory, &[conflicting]),
        Err(MergeFailure::ConflictingAttempt)
    );
}
