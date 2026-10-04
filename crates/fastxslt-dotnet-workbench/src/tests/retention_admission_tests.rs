//! Private result-heavy generation overlap controls; no ABI observation surface.

use std::{
    fmt::Write,
    sync::{Arc, Barrier},
};

use super::{
    ADMISSION_ENGINE_BYTES_EXHAUSTED, ADMISSION_TOTAL_BYTES_EXHAUSTED, ExperimentalEngine,
    OUTCOME_RESULT, Outcome, State, WorkbenchLimits, admission_status, policy_with,
    take_local_engine,
};

fn fixture(items: usize) -> (ExperimentalEngine, Vec<u8>) {
    let mut source = String::from("<root>");
    for index in 0..items {
        write!(
            source,
            "<item n=\"{index}\">{}&amp;&lt;</item>",
            "x".repeat(512)
        )
        .expect("write fixture");
    }
    source.push_str("</root>");
    let expected = source.as_bytes().to_vec();
    let stylesheet = br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:copy-of select="/root"/></xsl:template></xsl:stylesheet>"#;
    let engine = ExperimentalEngine::new(
        format!("urn:retention:source:{items}"),
        source.into_bytes(),
        "urn:retention:stylesheet".to_owned(),
        stylesheet.to_vec(),
        WorkbenchLimits::default(),
    )
    .expect("prepare result-heavy engine");
    assert_eq!(engine.transform_bytes("initial").unwrap(), expected);
    (engine, expected)
}

fn state(engine_limit: usize, total_limit: usize) -> State {
    let state = State::new();
    assert_eq!(
        state.configure_policy(policy_with(2, 0, 3, usize::MAX, engine_limit, total_limit,)),
        0
    );
    state
}

fn charges(state: &State, engines: usize, outcomes: usize, engine_bytes: usize, payload: usize) {
    assert_eq!(state.engines().unwrap().len(), engines);
    assert_eq!(state.outcomes().unwrap().len(), outcomes);
    let accounting = state.accounting().unwrap();
    assert_eq!(accounting.engine_known_capacity_bytes, engine_bytes);
    assert_eq!(accounting.outcome_payload_bytes, payload);
}

fn reusable(state: &State, handle: u64, expected: &[u8]) {
    let engine = state
        .engines()
        .unwrap()
        .get(&handle)
        .unwrap()
        .engine
        .clone();
    assert_eq!(engine.transform_bytes("reuse").unwrap(), expected);
}

#[test]
fn overlapping_result_heavy_engines_conserve_exact_and_one_less_capacity() {
    for items in [5, 50, 500] {
        for exact in [false, true] {
            let (old, expected_old) = fixture(5);
            let (new, expected_new) = fixture(items);
            let old_charge = old.retention_estimate().known_retained_capacity_bytes;
            let new_charge = new.retention_estimate().known_retained_capacity_bytes;
            let limit = old_charge + new_charge - usize::from(!exact);
            let state = state(limit, usize::MAX);
            let old_handle = take_local_engine(&state, state.insert_created_engine(old));
            let creation = state.insert_created_engine(new);
            if exact {
                let new_handle = take_local_engine(&state, creation);
                charges(&state, 2, 0, old_charge + new_charge, 0);
                reusable(&state, new_handle, &expected_new);
                assert!(state.release_engine(new_handle));
            } else {
                assert_eq!(creation, admission_status(ADMISSION_ENGINE_BYTES_EXHAUSTED));
                charges(&state, 1, 0, old_charge, 0);
                assert!(!state.release_outcome(creation));
                assert!(!state.release_engine(creation));
            }
            reusable(&state, old_handle, &expected_old);
            assert!(state.release_engine(old_handle));
            charges(&state, 0, 0, 0, 0);
            let (replacement, _) = fixture(items);
            let replacement = take_local_engine(&state, state.insert_created_engine(replacement));
            reusable(&state, replacement, &expected_new);
            assert!(state.release_engine(replacement));
            charges(&state, 0, 0, 0, 0);
        }
    }
}

#[test]
fn retained_result_payload_participates_in_atomic_generation_admission() {
    for items in [5, 50, 500] {
        for exact in [false, true] {
            let (old, expected_old) = fixture(5);
            let (new, expected_new) = fixture(items);
            let old_charge = old.retention_estimate().known_retained_capacity_bytes;
            let new_charge = new.retention_estimate().known_retained_capacity_bytes;
            let payload = expected_old.len();
            let state = state(
                usize::MAX,
                old_charge + new_charge + payload - usize::from(!exact),
            );
            let old_handle = take_local_engine(&state, state.insert_created_engine(old));
            let retained = state.insert_outcome(Outcome::Bytes {
                kind: OUTCOME_RESULT,
                value: expected_old.clone(),
            });
            charges(&state, 1, 1, old_charge, payload);
            let creation = state.insert_created_engine(new);
            if exact {
                let new_handle = take_local_engine(&state, creation);
                charges(&state, 2, 1, old_charge + new_charge, payload);
                reusable(&state, new_handle, &expected_new);
                assert!(state.release_engine(new_handle));
            } else {
                assert_eq!(creation, admission_status(ADMISSION_TOTAL_BYTES_EXHAUSTED));
                charges(&state, 1, 1, old_charge, payload);
            }
            reusable(&state, old_handle, &expected_old);
            assert!(state.release_engine(old_handle));
            charges(&state, 0, 1, 0, payload);
            match state.outcomes().unwrap().get(&retained).unwrap() {
                Outcome::Bytes { kind, value } => {
                    assert_eq!(*kind, OUTCOME_RESULT);
                    assert_eq!(value, &expected_old);
                }
                Outcome::Engine(_) => panic!("result ownership changed"),
            }
            let (replacement, _) = fixture(items);
            let replacement = take_local_engine(&state, state.insert_created_engine(replacement));
            reusable(&state, replacement, &expected_new);
            assert!(state.release_engine(replacement));
            assert!(state.release_outcome(retained));
            charges(&state, 0, 0, 0, 0);
        }
    }
}

#[test]
fn result_admission_total_limit_releases_without_evicting_its_engine() {
    for items in [5, 50, 500] {
        for exact in [false, true] {
            let (engine, expected) = fixture(items);
            let charge = engine.retention_estimate().known_retained_capacity_bytes;
            let state = state(usize::MAX, charge + expected.len() - usize::from(!exact));
            let handle = take_local_engine(&state, state.insert_created_engine(engine));
            let result = state.insert_outcome(Outcome::Bytes {
                kind: OUTCOME_RESULT,
                value: expected.clone(),
            });
            if exact {
                charges(&state, 1, 1, charge, expected.len());
                assert!(state.release_outcome(result));
            } else {
                assert_eq!(result, admission_status(ADMISSION_TOTAL_BYTES_EXHAUSTED));
            }
            charges(&state, 1, 0, charge, 0);
            reusable(&state, handle, &expected);
            assert!(state.release_engine(handle));
            let result = state.insert_outcome(Outcome::Bytes {
                kind: OUTCOME_RESULT,
                value: expected.clone(),
            });
            charges(&state, 0, 1, 0, expected.len());
            assert!(state.release_outcome(result));
            charges(&state, 0, 0, 0, 0);
        }
    }
}

#[test]
fn concurrent_last_generation_byte_admission_has_one_winner() {
    for items in [5, 50, 500] {
        for total_limit in [false, true] {
            let (old, expected_old) = fixture(5);
            let (first, expected_new) = fixture(items);
            let (second, _) = fixture(items);
            let old_charge = old.retention_estimate().known_retained_capacity_bytes;
            let new_charge = first.retention_estimate().known_retained_capacity_bytes;
            assert_eq!(
                second.retention_estimate().known_retained_capacity_bytes,
                new_charge
            );
            let limit = old_charge + new_charge;
            let state = Arc::new(State::new());
            assert_eq!(
                state.configure_policy(policy_with(
                    usize::MAX,
                    0,
                    usize::MAX,
                    usize::MAX,
                    if total_limit { usize::MAX } else { limit },
                    if total_limit { limit } else { usize::MAX },
                )),
                0
            );
            let old_handle = take_local_engine(&state, state.insert_created_engine(old));
            let barrier = Arc::new(Barrier::new(3));
            let threads: Vec<_> = [first, second]
                .into_iter()
                .map(|engine| {
                    let state = state.clone();
                    let barrier = barrier.clone();
                    std::thread::spawn(move || {
                        barrier.wait();
                        state.insert_created_engine(engine)
                    })
                })
                .collect();
            barrier.wait();
            let outcomes: Vec<_> = threads
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .collect();
            let denied = admission_status(if total_limit {
                ADMISSION_TOTAL_BYTES_EXHAUSTED
            } else {
                ADMISSION_ENGINE_BYTES_EXHAUSTED
            });
            assert_eq!(
                outcomes
                    .iter()
                    .filter(|&&outcome| outcome == denied)
                    .count(),
                1
            );
            charges(&state, 2, 1, limit, 0);
            let winner = *outcomes.iter().find(|&&outcome| outcome != denied).unwrap();
            let winner = take_local_engine(&state, winner);
            reusable(&state, old_handle, &expected_old);
            reusable(&state, winner, &expected_new);
            assert!(state.release_engine(winner));
            assert!(state.release_engine(old_handle));
            charges(&state, 0, 0, 0, 0);
        }
    }
}
