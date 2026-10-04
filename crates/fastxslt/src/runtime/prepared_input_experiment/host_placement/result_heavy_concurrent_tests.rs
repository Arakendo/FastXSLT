//! Concurrent invocation controls over independently owned immutable generations.

use super::super::StylesheetProgram;
use super::{
    CANDIDATES, Capacity, Document, InvocationControl, WorkDomain, construct, execute_program,
    input, program, runtime_case_control, serialize_xml,
};
use crate::runtime::golden_runtime_experiment::ExecutionFailure;
use std::sync::{Arc, Barrier};

const DOMAINS: [WorkDomain; 10] = [
    WorkDomain::XmlEvent,
    WorkDomain::XdmNode,
    WorkDomain::XdmStringValueNode,
    WorkDomain::XPathNodeVisit,
    WorkDomain::XPathOperation,
    WorkDomain::XsltInstruction,
    WorkDomain::XsltTemplateCandidate,
    WorkDomain::ResultNode,
    WorkDomain::ResultTextByte,
    WorkDomain::SerializedByte,
];

fn run(
    program: &StylesheetProgram,
    document: &Document,
    control: &mut InvocationControl,
    output_limit: usize,
) -> Result<String, ExecutionFailure> {
    execute_program(program, document, "concurrent-capacity", control).and_then(|result| {
        serialize_xml(
            &result,
            &program.output,
            "concurrent-capacity",
            output_limit,
            control,
        )
    })
}

fn exercise(
    program: &StylesheetProgram,
    reference: &Document,
    candidate: &Document,
    expected: &str,
    worker: usize,
) -> String {
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        run(program, reference, &mut baseline, expected.len()).unwrap(),
        expected
    );
    let (domain, case) = match worker {
        0 => (WorkDomain::ResultNode, 0),
        1 => (WorkDomain::SerializedByte, 1),
        2 => (WorkDomain::ResultNode, 2),
        3 => (WorkDomain::ResultTextByte, 1),
        _ => unreachable!(),
    };
    let mut reference_control = runtime_case_control(domain, baseline.consumed(domain), case);
    let mut candidate_control = runtime_case_control(domain, baseline.consumed(domain), case);
    let reference_result = run(program, reference, &mut reference_control, expected.len());
    let candidate_result = run(program, candidate, &mut candidate_control, expected.len());
    assert_eq!(candidate_result, reference_result);
    assert_eq!(candidate_result.is_ok(), case == 0);
    #[cfg(feature = "workbench")]
    if let Err(failure) = candidate_result {
        let (code, category, request, _, _) = failure.workbench_parts();
        assert_eq!(request, Some("concurrent-capacity"));
        assert_eq!(
            (code, category),
            if case == 2 {
                ("FXCT0001", "cancelled")
            } else {
                ("FXCT0002", "limit")
            }
        );
    }
    for domain in DOMAINS {
        assert_eq!(
            candidate_control.consumed(domain),
            reference_control.consumed(domain)
        );
    }
    // A failed invocation cannot poison the shared source or another invocation's control.
    let recovered = run(
        program,
        candidate,
        &mut InvocationControl::unbounded(),
        expected.len(),
    )
    .unwrap();
    assert_eq!(recovered, expected);
    recovered
}

#[test]
fn result_heavy_shared_candidates_isolate_controls_while_generation_leases_drain() {
    let program = Arc::new(program());
    for items in [5, 50, 500] {
        let inputs = [input(items, 8), input(items, 9)];
        let references = inputs.each_ref().map(|input| {
            Arc::new(
                construct(
                    input,
                    Capacity::Growth,
                    false,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap(),
            )
        });
        for candidate in CANDIDATES {
            for polls in [false, true] {
                let documents = inputs.each_ref().map(|input| {
                    Arc::new(
                        construct(input, candidate, polls, &mut InvocationControl::unbounded())
                            .unwrap(),
                    )
                });
                assert!(!documents[0].same_origin(&documents[1]));
                let weak = documents.each_ref().map(Arc::downgrade);
                let barrier = Arc::new(Barrier::new(5));
                let threads: Vec<_> = (0..4)
                    .map(|worker| {
                        let program = program.clone();
                        let barrier = barrier.clone();
                        let references = references.clone();
                        let documents = documents.clone();
                        let expected = inputs.each_ref().map(|input| input.expected.clone());
                        std::thread::spawn(move || {
                            barrier.wait();
                            let mut retained = Vec::new();
                            // Vary generation visit order; no completion-order requirement exists.
                            for generation in [worker % 2, 1 - worker % 2] {
                                let output = exercise(
                                    &program,
                                    &references[generation],
                                    &documents[generation],
                                    &expected[generation],
                                    worker,
                                );
                                retained.push((generation, output));
                            }
                            retained
                        })
                    })
                    .collect();
                // Retire both caller-owned generations before any worker begins execution.
                drop(documents);
                for owner in &weak {
                    assert_eq!(owner.strong_count(), 4);
                }
                barrier.wait();
                let retained: Vec<_> = threads
                    .into_iter()
                    .flat_map(|thread| thread.join().expect("independent invocation must finish"))
                    .collect();
                for owner in &weak {
                    assert!(owner.upgrade().is_none());
                }
                for (generation, output) in retained {
                    assert_eq!(output, inputs[generation].expected);
                }
            }
        }
    }
}
