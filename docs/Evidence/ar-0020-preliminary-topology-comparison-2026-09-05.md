# AR-0020 Preliminary Preparation-Topology Comparison

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Status | Preliminary private topology evidence |
| Governing review | [AR-0020](../Architectural%20Reviews/AR-0020-bounded-pre-execution-preparation-pipeline.md) |
| Workload | Pinned XSLT30 `for-004` stylesheet with deterministic 50- and 500-item raw sources |
| Batch | 256 independently prepared and semantically checked requests |
| Samples | Seven complete batches per topology; median batch reported; two runs |
| Queue | At most two ready packets and two packets' known prepared capacity |
| Execution | Exactly one transform worker in every pipelined topology |

## Lanes

- **A: direct sequential** prepares and executes each request on one thread.
- **B: pipeline one** uses one preparation worker feeding exactly one transform worker.
- **C: pipeline four** uses four preparation workers feeding exactly one transform worker.

Thread creation, preparation, queueing, execution, serialization, result
checking, and producer joining are inside each measured batch. Compiled
stylesheet construction and one capacity probe are outside it. Every result
must contain the expected `<out>N.00</out>` sentinel.

Command:

```powershell
cargo test --release -p fastxslt measures_direct_and_single_transform_worker_pipeline_topologies --all-features -- --ignored --nocapture
```

## Two-run results

| Items | Direct throughput | Pipeline one | One-preparer speedup | Pipeline four | Four-preparer speedup |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 50 | 15,587-15,801/s | 16,749-17,192/s | 1.06-1.10x | 41,954-42,660/s | 2.69-2.70x |
| 500 | 1,558-1,737/s | 1,921-1,987/s | 1.14-1.23x | 6,330-6,454/s | 3.64-4.14x |

| Items | One-preparer consumer wait | Four-preparer consumer wait | One-preparer total prepared high-water | Four-preparer total prepared high-water |
| ---: | ---: | ---: | ---: | ---: |
| 50 | 9.8-10.4 ms | 0.50-0.63 ms | 72,323-144,646 B | 216,969 B |
| 500 | 94.7-100.2 ms | 7.4-8.3 ms | 593,673 B | 1,781,019 B |

The total high-water includes ready and executing prepared documents. The
four-preparer maxima are exactly three packets: the two-packet ready envelope
plus the single actively executing packet. The one-preparer lane frequently
cannot fill the ready queue because preparation is its limiting stage.

## Interpretation

Cross-request overlap helps even with only one preparation worker, while four
preparers substantially reduce the transform worker's starvation on these raw
sources. This agrees with the preceding phase probe: one preparer remains the
bottleneck, whereas four can feed execution more effectively.

The gain is purchased with additional simultaneous prepared state. The bounded
queue prevents it from growing beyond the configured ready envelope, and the
observed total remains ready capacity plus one active packet. No result order
or semantic meaning is assigned to preparation order.

## Claim boundary

This is a private same-process Rust experiment with generated pressure sources,
not an ASP.NET result, representative publication trace, selected public
executor, or default worker-count recommendation. It does not yet cover mixed
document sizes, shared-source reuse, cancellation/failure while producers are
blocked, generation replacement, queue teardown, or the already-warm negative
control. Those remain required before AR-0020 can nominate a topology.
