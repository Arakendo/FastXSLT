//! Manual reference economics; not compiled engine or host throughput.

use super::*;
use std::hint::black_box;
use std::time::Instant;

fn elapsed(iterations: usize, mut action: impl FnMut()) -> f64 {
    let start = Instant::now();
    let mut completed = 0;
    loop {
        for _ in 0..iterations {
            action();
        }
        completed += iterations;
        if start.elapsed() >= std::time::Duration::from_millis(20) {
            break;
        }
    }
    start.elapsed().as_secs_f64() * 1_000_000_000.0
        / f64::from(u32::try_from(completed).expect("bounded probe iteration count"))
}

#[test]
#[ignore = "manual release owner reference timing; omit allocation-observation"]
fn retained_owner_reference_timing() {
    for owner_count in [1, 16, 64] {
        let owners = (0..owner_count)
            .map(|_| super::super::tests::document(b"<r><a/></r>"))
            .collect::<Vec<_>>();
        for node_count in [1, 16, 1024] {
            let ids = vec![super::super::tests::root(&owners[0]); node_count];
            let mut table = OwnerTable::default();
            let keys = owners
                .iter()
                .map(|owner| {
                    table
                        .admit(owner, owner_count, &mut InvocationControl::unbounded())
                        .unwrap()
                })
                .collect::<Vec<_>>();
            let explicit = retain(
                &owners[owner_count - 1],
                &ids,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let selected = table
                .select(
                    &keys[owner_count - 1],
                    &ids,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
            let iterations = (100_000 / node_count).max(100);
            for trial in 0..5 {
                // Rotate paired timing order; setup is intentionally outside lookup/clone.
                for lane in if trial % 2 == 0 { [0, 1] } else { [1, 0] } {
                    let construct = elapsed(iterations, || {
                        if lane == 0 {
                            black_box(
                                retain(
                                    black_box(&owners[owner_count - 1]),
                                    black_box(&ids),
                                    &mut InvocationControl::unbounded(),
                                )
                                .unwrap(),
                            );
                        } else {
                            black_box(
                                table
                                    .select(
                                        black_box(&keys[owner_count - 1]),
                                        black_box(&ids),
                                        &mut InvocationControl::unbounded(),
                                    )
                                    .unwrap(),
                            );
                        }
                    });
                    let lookup = elapsed(iterations, || {
                        for index in 0..node_count {
                            if lane == 0 {
                                black_box(black_box(&explicit)[index].borrowed());
                            } else {
                                black_box(black_box(&selected).borrowed(index));
                            }
                        }
                    });
                    let clone_drop = elapsed(iterations, || {
                        if lane == 0 {
                            black_box(black_box(&explicit).clone());
                        } else {
                            black_box(TableSelection {
                                table: selected.table,
                                nodes: black_box(&selected.nodes).clone(),
                            });
                        }
                    });
                    println!(
                        "owners={owner_count} nodes={node_count} trial={trial} lane={} construct_drop_ns={construct:.1} lookup_all_ns={lookup:.1} clone_drop_ns={clone_drop:.1}",
                        if lane == 0 { "explicit" } else { "table" }
                    );
                }
            }
            drop(selected);
            for trial in 0..5 {
                let admission = elapsed(iterations, || {
                    black_box(
                        table
                            .admit(
                                black_box(&owners[owner_count - 1]),
                                owner_count,
                                &mut InvocationControl::unbounded(),
                            )
                            .unwrap(),
                    );
                });
                println!(
                    "owners={owner_count} nodes={node_count} trial={trial} repeated_last_owner_admit_ns={admission:.1}"
                );
            }
        }
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
fn retained_owner_reference_allocation_scopes_release_every_allocation() {
    for owner_count in [1, 16, 64] {
        let owners = (0..owner_count)
            .map(|_| super::super::tests::document(b"<r><a/></r>"))
            .collect::<Vec<_>>();
        for node_count in [1, 16, 1024] {
            let ids = vec![super::super::tests::root(&owners[0]); node_count];
            for lane in [0, 1] {
                let info = allocation_counter::measure(|| {
                    if lane == 0 {
                        let values = owners
                            .iter()
                            .map(|owner| {
                                retain(owner, &ids, &mut InvocationControl::unbounded()).unwrap()
                            })
                            .collect::<Vec<_>>();
                        black_box(&values);
                    } else {
                        let mut table = OwnerTable::default();
                        let keys = owners
                            .iter()
                            .map(|owner| {
                                table
                                    .admit(owner, owner_count, &mut InvocationControl::unbounded())
                                    .unwrap()
                            })
                            .collect::<Vec<_>>();
                        let values = keys
                            .iter()
                            .map(|key| {
                                table
                                    .select(key, &ids, &mut InvocationControl::unbounded())
                                    .unwrap()
                            })
                            .collect::<Vec<_>>();
                        black_box(&values);
                    }
                });
                assert_eq!(info.bytes_current, 0);
                assert_eq!(info.count_current, 0);
                println!(
                    "owners={owner_count} nodes={node_count} lane={} requested_total={} requested_peak={} allocations={}",
                    if lane == 0 { "explicit" } else { "table" },
                    info.bytes_total,
                    info.bytes_max,
                    info.count_total
                );
            }
            assert!(owners.iter().all(|owner| Arc::strong_count(owner) == 1));
        }
    }
}
