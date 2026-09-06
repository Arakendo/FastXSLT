# AR-0020 Equal-Thread-Budget Comparison

| Field               | Value                                                                                       |
| ------------------- | ------------------------------------------------------------------------------------------- |
| Date                | 2026-09-05                                                                                  |
| Status              | Preliminary private topology evidence                                                       |
| Governing review    | [AR-0020](../Architectural%20Reviews/AR-0020-bounded-pre-execution-preparation-pipeline.md) |
| Workload            | Pinned XSLT30 `for-004` stylesheet with deterministic 50- and 500-item raw sources          |
| Batch               | 256 independently prepared and semantically checked requests                                |
| Samples             | Five complete batches per topology; median batch reported; two runs                         |
| Total thread budget | Ten worker threads in every lane                                                            |
| Ready envelope      | At most ten packets and ten packets' known prepared capacity                                |

## Question and method

Does a staged preparation/execution pool outperform combined workers when both
receive the same total thread budget?

The control uses ten combined workers, each performing prepare then execute.
The staged alternatives allocate the same ten threads as 9/1, 8/2, 7/3, and
5/5 preparation/execution workers. Thread startup, preparation, handoff,
execution, serialization, semantic checking, and joining are included. The
compiled stylesheet and one prepared-capacity probe are outside the measured
batch.

Command:

```powershell
cargo test --release -p fastxslt measures_equal_ten_thread_combined_and_staged_topologies --all-features -- --ignored --nocapture
```

## Throughput

| Topology              |  50-item throughput | 500-item throughput |
| --------------------- | ------------------: | ------------------: |
| 10 combined           |     42,274-44,809/s | **10,472-11,353/s** |
| 9 prepare / 1 execute |     31,980-35,081/s |       6,967-7,655/s |
| 8 prepare / 2 execute |     54,843-63,396/s |     10,072-10,628/s |
| 7 prepare / 3 execute | **50,636-64,144/s** |      9,734-10,061/s |
| 5 prepare / 5 execute |     47,327-56,436/s |       8,185-8,534/s |

At 50 items, 8/2 and 7/3 both beat the combined control in both runs, although
their ranking against each other varied. At 500 items, the combined topology
won both runs. The closest staged allocation was 8/2, roughly 4-6% behind the
combined control.

The 9/1 allocation is not favored merely because the isolated phase ratio
suggests preparation-heavy allocation. One execution worker becomes the
bottleneck after parallel preparation and pays the handoff cost. The 5/5 split
overprovisions execution for these preparation-heavy inputs.

## Latency and retention observations

| Items / topology |  p50 range |      p95 range |      p99 range | Prepared high-water range |
| ---------------- | ---------: | -------------: | -------------: | ------------------------: |
| 50 / combined    | 199-209 us |     355-388 us |     415-472 us |         361,615-506,261 B |
| 50 / 8/2         | 155-203 us |     338-370 us |     391-730 us |         650,907-867,876 B |
| 50 / 7/3         | 113-142 us |     290-304 us |     342-350 us |         433,938-578,584 B |
| 500 / combined   | 819-964 us | 1,288-1,350 us | 1,499-1,597 us |     3,562,038-4,749,384 B |
| 500 / 8/2        | 944-965 us | 1,328-1,424 us | 1,475-1,768 us |     4,155,711-4,749,384 B |

The table retains the strongest contenders rather than hiding latency and
memory behind throughput. Staged 50-item wins generally retain more prepared
capacity than the combined control. At 500 items the combined lane provides
both higher throughput and generally better latency than 8/2.

Aggregate executor wait is not comparable as a utilization percentage across
different executor counts: it sums time across every execution thread. It is
still directionally useful within a configuration and remains in raw probe
output. A later harness should report per-worker busy/wait distributions.

## Disposition

Equal thread budget invalidates any universal claim that staging wins. The
current evidence instead shows a workload-shape decision surface: staging helps
the medium source, while combined locality wins on the larger source. AR-0020
must therefore retain combined workers as a real candidate and must not select
one fixed split from the earlier phase ratio alone.

This remains a same-process generated-source experiment. It does not choose an
adaptive scheduler, public topology, default thread count, or ASP.NET behavior.
Representative mixed-size batches, host limits, failure/cancellation teardown,
and the warm negative control remain outstanding.
