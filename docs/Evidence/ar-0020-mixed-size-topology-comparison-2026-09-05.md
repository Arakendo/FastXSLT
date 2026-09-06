# AR-0020 Mixed-Size Topology Comparison

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Status | Preliminary private workload-shape evidence |
| Governing review | [AR-0020](../Architectural%20Reviews/AR-0020-bounded-pre-execution-preparation-pipeline.md) |
| Workload | Pinned XSLT30 `for-004` stylesheet; generated 5-, 50-, and 500-item raw sources |
| Distribution | 2,048 requests: 1,024 small, 768 medium, 256 large (4:3:1) |
| Orders | Size-clustered and deterministic repeated small/medium/large interleave |
| Samples | Seven complete batches per topology; median batch reported; two runs |
| Thread budget | Ten combined, staged 8/2, or staged 7/3 |
| Ready envelope | Ten packets and three large-packet equivalents of prepared capacity |

## Question

Do staged pools smooth a heterogeneous batch better than combined workers, and
does submission order alter the answer?

Every request independently prepares its source and must produce the expected
size-specific result. Thread startup, preparation, queueing, execution,
serialization, checking, and joining are included. Prepared source snapshots
and the compiled stylesheet are constructed before timing. The distribution is
an explicit exploratory stress mix, not a measured consumer distribution.

Command:

```powershell
cargo test --release -p fastxslt measures_clustered_and_interleaved_mixed_size_batches --all-features -- --ignored --nocapture
```

## Throughput

| Order / topology | Throughput range |
| --- | ---: |
| Clustered / 10 combined | **49,701-51,939/s** |
| Clustered / staged 8/2 | 48,733-49,150/s |
| Clustered / staged 7/3 | 48,925-50,451/s |
| Interleaved / 10 combined | **58,522-58,772/s** |
| Interleaved / staged 8/2 | 54,126-56,284/s |
| Interleaved / staged 7/3 | 51,831-55,186/s |

Combined workers led both interleaved runs and one clustered run. Staged 7/3
led the other clustered run by only about 1.5%, then trailed by about 5.8% in
the repeat. Staging therefore did not establish a repeatable mixed-batch
throughput advantage.

Interleaving improved all lanes. Combined throughput rose roughly 13-18% over
its clustered runs. The likely load-balancing explanation is an inference;
this probe does not measure cache misses or scheduler placement.

## Per-size p95 latency

| Order / topology | Small | Medium | Large |
| --- | ---: | ---: | ---: |
| Clustered / combined | 18-22 us | 168-194 us | 1,657-1,670 us |
| Clustered / 8/2 | 109-123 us | 295-323 us | 1,478-1,602 us |
| Clustered / 7/3 | 78-103 us | 159-162 us | 1,177-1,452 us |
| Interleaved / combined | 23-24 us | 137-141 us | 1,172-1,340 us |
| Interleaved / 8/2 | 240-261 us | 324-344 us | 1,274-1,339 us |
| Interleaved / 7/3 | 103-110 us | 175-199 us | 1,120-1,194 us |

Staged 7/3 can improve the large-request tail, especially in the clustered
batch, but makes small-request p95 several times worse. It is therefore a
latency-allocation tradeoff rather than a smoothing win. Aggregate latency
would hide this result.

Prepared-capacity high-water ranged from about 2.52-4.75 MB for combined,
2.70-2.97 MB for 8/2, and 2.71-3.56 MB for 7/3. The staged ready queue remained
bounded by count and its fixed three-large-packet byte envelope; reported total
also includes actively executing packets.

## Disposition

For this exploratory mix, combined workers are the more robust throughput
candidate. Submission order materially affects every topology, and 7/3 offers
a possible host-selected large-tail/small-tail tradeoff rather than a universal
improvement. AR-0020 must retain per-size latency and order/distribution as
selection inputs.

This does not select combined workers as the product default. Consumer source
distributions, trust/latency goals, ASP.NET boundary costs, cancellation and
teardown, and the warm negative control remain missing.
