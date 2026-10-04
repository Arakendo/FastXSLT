//! Candidate handoff observations, not an atomic host-publication contract.

use super::{
    CANDIDATES, CancellationToken, Capacity, CapacityCheckpoint, Document, InvocationControl,
    ParseLimits, WorkLimits, construct, execute_program, input, parse_document_controlled, program,
    serialize_xml,
};
use std::sync::Arc;

fn output(program: &super::super::StylesheetProgram, document: &Document, expected: &str) {
    let mut control = InvocationControl::unbounded();
    let result = execute_program(program, document, "publication-control", &mut control).unwrap();
    assert_eq!(
        serialize_xml(
            &result,
            &program.output,
            "publication-control",
            expected.len(),
            &mut control
        )
        .unwrap(),
        expected
    );
}

#[test]
fn cancelled_replacement_keeps_old_leases_and_source_generation_intact() {
    let program = program();
    for items in [5, 50, 500] {
        let old_input = input(items, 8);
        let new_input = input(items, 9);
        for freeze in [false, true] {
            for after in [false, true] {
                let old = Arc::new(
                    construct(
                        &old_input,
                        Capacity::Growth,
                        false,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap(),
                );
                let lease = Arc::clone(&old);
                let weak = Arc::downgrade(&old);
                let mut published = old;
                let parsed = parse_document_controlled(
                    &new_input.identity,
                    &new_input.source,
                    ParseLimits {
                        max_events: 10_000,
                        max_depth: 64,
                    },
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                let token = CancellationToken::new();
                let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
                let mut signalled = false;
                let mut observer = |checkpoint| {
                    let before_resize = matches!(
                        checkpoint,
                        CapacityCheckpoint::BeforeReserve { .. }
                            | CapacityCheckpoint::BeforeFreeze { .. }
                    );
                    let after_resize = matches!(
                        checkpoint,
                        CapacityCheckpoint::AfterReserve { .. }
                            | CapacityCheckpoint::AfterFreeze { .. }
                    );
                    if (!after && before_resize) || (after && after_resize) {
                        signalled = true;
                        token.cancel();
                    }
                };
                let replacement = Document::from_parsed_with_capacity_polls(
                    parsed,
                    &mut control,
                    !freeze,
                    freeze,
                    Some(&mut observer),
                );
                assert!(signalled && replacement.is_err());
                // This test caller publishes only successful construction, not partial state.
                if let Ok(document) = replacement {
                    published = Arc::new(document);
                }
                assert!(Arc::ptr_eq(&published, &lease));
                assert_eq!(Arc::strong_count(&published), 2);
                output(&program, &published, &old_input.expected);
                let fresh = Arc::new(
                    construct(
                        &new_input,
                        if freeze {
                            Capacity::Frozen
                        } else {
                            Capacity::Presized
                        },
                        true,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap(),
                );
                assert!(!fresh.same_origin(&lease));
                published = fresh;
                output(&program, &lease, &old_input.expected);
                output(&program, &published, &new_input.expected);
                drop(lease);
                assert!(weak.upgrade().is_none());
                output(&program, &published, &new_input.expected);
            }
        }
    }
}

#[test]
fn cancellation_after_constructor_return_does_not_revoke_immutable_source() {
    let program = program();
    for items in [5, 50, 500] {
        let input = input(items, 8);
        for candidate in CANDIDATES {
            for polls in [false, true] {
                let token = CancellationToken::new();
                let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
                let document = construct(&input, candidate, polls, &mut control).unwrap();
                // Deterministic signal after return, before this caller shares the owner.
                token.cancel();
                let published = Arc::new(document);
                let failure =
                    execute_program(&program, &published, "publication-control", &mut control)
                        .err()
                        .unwrap();
                #[cfg(feature = "workbench")]
                {
                    let (code, category, request, _, _) = failure.workbench_parts();
                    assert_eq!(
                        (code, category, request),
                        ("FXCT0001", "cancelled", Some("publication-control"))
                    );
                }
                drop(failure);
                // Reusing the cancelled operation control fails; a new invocation still works.
                output(&program, &published, &input.expected);
                let lease = Arc::clone(&published);
                let weak = Arc::downgrade(&published);
                drop(published);
                output(&program, &lease, &input.expected);
                drop(lease);
                assert!(weak.upgrade().is_none());
            }
        }
    }
}
