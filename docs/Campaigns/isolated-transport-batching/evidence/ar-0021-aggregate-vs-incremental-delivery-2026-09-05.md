# AR-0021 Aggregate versus Incremental Input-Order Delivery

| Field             | Value                                                                                                  |
| ----------------- | ------------------------------------------------------------------------------------------------------ |
| Date              | 2026-09-05                                                                                             |
| Source checkpoint | Working tree above `10d2f30`                                                                           |
| Host              | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0`                                     |
| Runtime           | .NET 10 preview workbench; release Rust worker                                                         |
| Workload          | Existing result-heavy 100/1,000/5,000-element fixtures                                                 |
| Members per cell  | 256                                                                                                    |
| Repetitions       | Three; batch and delivery order rotated                                                                |
| Claim             | Private delivery-mode comparison; not an accepted worker protocol or public streaming-results contract |

## Question

Aggregate input-order framing amortizes transport but withholds every outcome
until the last member completes. This experiment retains one sequential worker
lane and the same member order, identities, semantics, command/member limits,
and cumulative 1 MiB response ceiling, but flushes each complete correlated
outcome before executing the next member.

The incremental response begins with an acknowledged member count, emits an
indexed input-order outcome frame for every member, and ends with a counted
terminal frame. It does not introduce completion-order delivery or concurrent
transforms.

## Representative medians

|  Result | Workers | Batch | Delivery    | Throughput | First p50 | Final p50 |
| ------: | ------: | ----: | ----------- | ---------: | --------: | --------: |
| 3.3 KiB |       4 |    32 | aggregate   |   24,675/s |  4.425 ms |  4.425 ms |
| 3.3 KiB |       4 |    32 | incremental |   28,299/s |  0.219 ms |  4.366 ms |
| 3.3 KiB |       8 |    32 | aggregate   |   40,756/s |  5.365 ms |  5.365 ms |
| 3.3 KiB |       8 |    32 | incremental |   40,811/s |  0.341 ms |  5.309 ms |
|  33 KiB |       4 |    16 | aggregate   |    2,871/s | 21.373 ms | 21.373 ms |
|  33 KiB |       4 |    16 | incremental |    3,030/s |  1.274 ms | 20.751 ms |
|  33 KiB |       8 |    16 | aggregate   |    4,850/s | 24.533 ms | 24.533 ms |
|  33 KiB |       8 |    16 | incremental |    4,904/s |  1.696 ms | 25.115 ms |
| 165 KiB |       4 |     4 | aggregate   |      575/s | 26.685 ms | 26.685 ms |
| 165 KiB |       4 |     4 | incremental |      560/s |  6.820 ms | 27.544 ms |
| 165 KiB |       8 |     4 | aggregate   |      676/s | 30.407 ms | 30.407 ms |
| 165 KiB |       8 |     4 | incremental |      706/s |  8.643 ms | 34.455 ms |

Incremental delivery moves first-result latency close to one-member service
time. The representative improvement is about 16-20x for 3.3/33 KiB results
at the listed multi-worker batches and about 3.5-3.9x for 165 KiB results.

Final throughput is generally in the same region. Some cells improved and some
regressed: for example, single-worker 33 KiB batch 16 fell from 1,419/s to
1,122/s, while four-worker 3.3 KiB batch 32 rose from 24,675/s to 28,299/s.
The short-cell ranges remain wide enough that those differences are not general
claims. The important repeatable distinction is early observation, not a
throughput win.

## Cost and conservation

Incremental framing adds an acknowledgement, an index per outcome, a terminal
frame, and one flush per member. Managed allocation rose by roughly 0.6-1.3
KiB per member in the observed cells. That is visible for the 3.3 KiB result
and proportionally small beside the 100-498 KiB result-materialization cost of
the larger fixtures. Working-set observations did not show a repeatable winner.

A focused Rust protocol test and the ASP.NET operational gate execute the same
success / pre-cancelled / zero-instruction-budget / success sequence through
aggregate and incremental paths. Both preserve input-order correlation, exact
`FXCT0001` and `FXCT0002` fields, successful later siblings, one sequential
execution lane, and subsequent worker reuse. The existing partial-transfer
fault probe uses the same incremental outcome shape and preserves a fully
observed prefix across later worker loss.

## Consumer boundary probe

The private client now also has an experimental callback-consumption path. It
invokes the consumer after each complete correlated outcome and does not retain
a completed-outcome collection in the adapter. In one operational run, four
exact 165,011-byte outcomes produced four callbacks and zero retained adapter
outcomes; first observation occurred after 6.997 ms and terminal completion
after 23.131 ms. The same worker then completed an exact recovery transform.

The same gate independently recomputes exact encoded accounting. The four-
member command is 109 bytes and its successful response sequence is 660,198
bytes. Non-retaining consumption peaks at one 165,047-byte framed outcome;
collecting the same incremental outcomes peaks at four outcomes and 660,188
framed outcome bytes. The remaining ten response bytes are the acknowledgement
and terminal frames. These are exact wire-equivalent ownership charges, not
claims about CLR object size, allocator retention, pipe buffers, or process RSS.

A second trial deliberately threw from the callback after observing two exact
outcomes. Because unread response frames make that pipe connection
desynchronized, the adapter terminated the worker, did not retry any member,
and required a fresh worker. The replacement completed an exact transform. The
host remains responsible for any durable record of the already observed
prefix.

A third trial killed the worker from the first completed callback. The client
returned a typed transport-loss observation recording acknowledgement, one
complete host-observed outcome, and three remaining ambiguous members. It did
not call the remainder unstarted: without a surviving worker observation, some
later work could have completed into pipe buffers before process loss. The
non-retaining adapter carried no result collection in the exception, performed
no retry, and the separately created replacement worker completed exactly.
The typed loss observation also retains the exact command bytes, fully decoded
response-frame bytes, and adapter outcome-retention high-water reached before
loss. It does not mislabel a partially read frame as a complete observation.

This proves that early visibility and bounded adapter retention can reach a
real consumer boundary. It does not select callbacks over async enumeration,
stabilize abandonment/disposal semantics, or define how a supported API returns
the ambiguous and unstarted suffix after consumer abandonment. Those remain
product/protocol questions.

The cumulative-limit case is now executable. Eight 165,011-byte outcomes exceed
the 1 MiB transaction ceiling. The worker transfers six complete correlated
outcomes, executes member six but rejects its untransferable frame with exact
`FXWB1004 / limit`, and does not start member seven. The managed adapter retains
the six exact outcomes and reports member six as ambiguous plus member seven as
unstarted. No member is retried, and the same worker executes an exact recovery
transform. This establishes a prefix-preserving terminal-failure shape without
making it public.

The aggregate oracle previously checked the same 1 MiB ceiling only after all
member results had entered its completed-outcome vector. It now accounts each
encoded outcome before retention, keeps at most the fitting prefix, rejects the
first over-limit outcome with `FXWB1004 / limit`, and does not execute the
suffix. A focused Rust test retains six representative 165,011-byte outcomes
and rejects the seventh without changing the retained byte count. The ASP.NET
gate observes the exact aggregate limit failure and subsequent same-worker
recovery. Aggregate framing still exposes no completed prefix to the host; this
change bounds private retention rather than improving its loss classification.

A slow-consumer probe then returned four 165,011-byte outcomes while the managed
reader paused 30 ms after each of the first three. The first complete outcome
arrived after 7.009 ms and final completion after 97.016 ms, demonstrating that
consumer delay propagates through the incremental transaction. The worker
executes and writes one member at a time and retains no engine-side vector of
completed outcomes; pipe buffering and the host-owned outcome collection remain
bounded by the protocol and caller. The same worker completed an exact recovery.

Observed worker working set was 5,820,416 bytes before, peaked at 11,960,320,
and ended at 6,942,720. One allocator/process sample cannot attribute that peak
or establish a general bound, so it remains an operational observation rather
than an ownership claim.

## Disposition

Incremental input-order framing earns retention as a private AR-0021 candidate:
it materially reduces time to first result without a consistent final-
throughput penalty and preserves the controlled member semantics exercised so
far. It does not replace the simpler aggregate oracle and is not yet suitable
for a supported host contract. Completion-order delivery remains unjustified.
