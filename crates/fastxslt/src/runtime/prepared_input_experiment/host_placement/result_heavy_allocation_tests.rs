//! Fresh complete prefix scopes; allocation counters are never nested.

use super::{Input, StylesheetProgram, input, prepare, program};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
use allocation_counter::{AllocationInfo, measure};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Prepared,
    Result,
    Serialized,
    Released,
    CancelledCopy,
    LimitedOutput,
}

fn scope(program: &StylesheetProgram, input: &Input, stage: Stage) -> AllocationInfo {
    let mut retained = None;
    let info = measure(|| {
        let prepared = prepare(input);
        let document = prepared.get(&input.identity).unwrap();
        let mut control = InvocationControl::unbounded();
        if stage == Stage::CancelledCopy {
            control = control.cancelling_on_charge(WorkDomain::ResultNode, 2);
            let failure = execute_program(program, &document, "allocation-cancel", &mut control)
                .err()
                .unwrap();
            #[cfg(feature = "workbench")]
            {
                let (code, category, request, _, _) = failure.workbench_parts();
                assert_eq!(
                    (code, category, request),
                    ("FXCT0001", "cancelled", Some("allocation-cancel"))
                );
            }
            drop(failure);
            return;
        }
        let result = (stage != Stage::Prepared)
            .then(|| execute_program(program, &document, "allocation-copy", &mut control).unwrap());
        let output = if matches!(
            stage,
            Stage::Serialized | Stage::Released | Stage::LimitedOutput
        ) {
            let limit = if stage == Stage::LimitedOutput {
                input.expected.len() - 1
            } else {
                input.expected.len()
            };
            let value = serialize_xml(
                result.as_ref().unwrap(),
                &program.output,
                "allocation-copy",
                limit,
                &mut control,
            );
            if stage == Stage::LimitedOutput {
                let failure = value.err().unwrap();
                #[cfg(feature = "workbench")]
                {
                    let (code, category, request, _, _) = failure.workbench_parts();
                    assert_eq!(
                        (code, category, request),
                        ("FXSR0002", "limit", Some("allocation-copy"))
                    );
                }
                drop(failure);
                return;
            }
            let output = value.unwrap();
            assert_eq!(output, input.expected);
            Some(output)
        } else {
            None
        };
        if matches!(stage, Stage::Prepared | Stage::Result | Stage::Serialized) {
            retained = Some((prepared, document, result, output));
        }
    });
    if matches!(
        stage,
        Stage::Released | Stage::CancelledCopy | Stage::LimitedOutput
    ) {
        assert_eq!(info.count_current, 0);
        assert_eq!(info.bytes_current, 0);
    } else {
        assert!(info.bytes_current > 0);
    }
    drop(retained);
    info
}

fn run(print: bool) {
    let program = program();
    for items in [5, 50, 500] {
        let input = input(items, 8);
        for stage in [
            Stage::Prepared,
            Stage::Result,
            Stage::Serialized,
            Stage::Released,
            Stage::CancelledCopy,
            Stage::LimitedOutput,
        ] {
            let info = scope(&program, &input, stage);
            if print {
                println!(
                    "ar0027-result-allocation items={items} stage={stage:?} source_bytes={} result_bytes={} allocations={info:?}",
                    input.source.len(),
                    input.expected.len()
                );
            }
        }
        // The compiled program remains reusable after failed invocations.
        super::sample(&program, &input, 1);
    }
}

#[test]
fn result_heavy_scopes_release_success_cancelled_copy_and_limited_output() {
    run(false);
}

#[test]
#[ignore = "manual release result-heavy ownership and peak allocation probe; not RSS"]
fn measure_result_heavy_allocations() {
    run(true);
}
