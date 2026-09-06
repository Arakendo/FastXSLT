# AR-0021 Isolated Boundary Attribution and Batch Prototype

| Field              | Value                                                                                                                       |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| Date               | 2026-09-05                                                                                                                  |
| Source checkpoint  | Working tree above `10d2f30`                                                                                                |
| Host               | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0`                                                          |
| Toolchain          | Rust 1.95.0; .NET SDK/runtime 10 preview used by the workbench                                                              |
| Stylesheet         | Pinned XSLT30 `for-004`                                                                                                     |
| Source tiers       | 5, 50, and 500 deterministic `order-item` elements                                                                          |
| Execution topology | One persistent isolated worker; exactly one sequential transform lane                                                       |
| Claim              | Private attribution and transport-batching evidence; not a supported protocol, batch size, or product performance guarantee |

## Question

Does the roughly fixed warm isolated-worker premium primarily surround semantic
execution, and can one bounded multi-request transport transaction amortize
enough of that premium to justify continuing AR-0021?

## Attribution method

A private measured transform operation retains the ordinary transform as its
oracle. The .NET client records elapsed durations for gate acquisition, request
encoding/write, request flush, response wait/read, and the complete instrumented
call. The worker independently records elapsed command decode, decoded-event
queue residence, and engine execution including serialization.

Only durations cross the process boundary. The experiment does not subtract
host timestamps from worker timestamps. The reported unattributed round-trip is
the difference between independently measured durations and therefore includes
worker result framing/write/flush, pipe transit and wakeups, managed response
reads, and probe overhead. It is localization evidence, not an additive latency
account.

Command:

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -TargetFramework net10.0 `
  -IsolatedBoundaryBreakdown `
  -TieredRequests 250 `
  -MeasurementRuns 3
```

Medians of three reports, in microseconds per request:

| Tier      | Ordinary mean | Request write | Flush | Worker decode | Worker queue | Worker execute | Unattributed round trip | Instrumented total |
| --------- | ------------: | ------------: | ----: | ------------: | -----------: | -------------: | ----------------------: | -----------------: |
| 5 items   |        46.164 |        10.731 | 0.260 |         4.160 |        5.986 |          5.153 |                  28.958 |             53.657 |
| 50 items  |        50.913 |        10.716 | 0.220 |         4.028 |        5.837 |          8.557 |                  29.663 |             59.018 |
| 500 items |        82.556 |        10.792 | 0.242 |         3.963 |        6.310 |         39.024 |                  32.267 |             92.826 |

The instrumented operation is slower than the ordinary path and its parts must
not be added to the ordinary mean. The shape is nevertheless clear: request
emission, worker decode/queue, and the unattributed response/pipe/wakeup region
are material and nearly flat while worker execution grows with source size.
That result admits the transport-amortization prototype; it does not yet justify
rewriting the underlying pipe protocol.

## Bounded batch prototype

The private worker protocol now accepts one command containing 1 through 128
request identities. Command bytes are capped at 1 MiB. The worker executes every
member sequentially through the same retained engine, stores correlated member
outcomes, and emits one input-order response capped at 1 MiB. No preparation
stage, concurrent execution lane, retry, sibling visibility, or resource
authority was added.

A focused Rust test sends success, invalid-empty-identity, and later success
members in one command. The invalid member retains `FXWB0003`; both siblings
retain exact results. The measured-command framing also has a focused result and
duration-field test.

A second private controlled command adds per-member pre-cancellation or an
optional XSLT-instruction limit without changing the ordinary throughput frame.
It rejects a member that attempts to combine those controls rather than silently
ignoring one. A focused Rust test and the real ASP.NET operational suite execute:

```text
success
pre-cancelled -> FXCT0001 / cancelled
zero-instruction budget -> FXCT0002 / limit
later success
ordinary recovery success
```

The exact member identities survive every outcome and the same worker remains
usable. This establishes independent pre-dispatch controls only. It does not
establish active mid-member cancellation, deadline behavior, or partial-loss
classification.

## Aggregate-response loss classification

A third private operation exists only for fault injection. It retains the same
bounded member frame and sequential execution lane, acknowledges the admitted
batch, and emits bounded `started` and `finished` observations around each
member. It deliberately parks after the selected member's `started`
observation so the ASP.NET supervisor can kill the worker at a deterministic
first, middle, or last position. These observations are experiment telemetry,
not member results or an incremental production protocol.

The .NET 10 operational run exercised five-member batches parked at indices 0,
2, and 4. Each observed prefix was exact: members before the barrier reported
`started` then `finished`, the selected member reported `started`, and later
members emitted no observation. The host then terminated the process, created
a new worker, and completed a separately identified recovery transform. No
member from a killed batch was reissued.

The experiment establishes a deliberately conservative classification for the
aggregate-response oracle:

```text
worker-finished member, no correlated outcome transferred -> operationally ambiguous
started member, no correlated outcome transferred         -> operationally ambiguous
no start observation under the sequential probe           -> unstarted
fully transferred correlated outcome                       -> complete
```

Worker-local `finished` is not host-observed semantic completion because the
aggregate protocol has transferred no member outcome yet. This distinction is
the central result: batching cannot promote a computed-but-untransferred result
to `complete`. The fault probe also proves that a sequential acknowledged
suffix can be classified `unstarted` when a trusted bounded observation prefix
is available.

A separate fault-only transfer probe then executed five successful members. It
fully emitted correlated outcomes for members 0 and 1, emitted the result tag,
index, identity, declared byte length, and exactly half the result bytes for
member 2, then parked. The host consumed that exact prefix before terminating
the worker. It classified members 0 and 1 `complete`, member 2
`operationally-ambiguous`, and members 3 and 4 `unstarted`; it did not reissue
any killed attempt and completed a new recovery identity on a fresh worker.

This proves both that a fully received correlated outcome remains complete
after later worker death and that a partial result frame must not be promoted
to completion. It also makes the framing cost visible: transferring outcomes
incrementally creates more precise failure states. The probe is not an admitted
incremental-result mode and supplies no performance evidence for one.

A following framing run closed the pre-acknowledgement and malformed-command
distinctions without pretending they are
the same state:

- a command deliberately closed halfway through its second declared identity
  caused a nonzero worker exit before `read_command` could publish any batch to
  the supervisor; all members are therefore unstarted and none were admitted;
- a complete valid batch was written and flushed, but the host deliberately
  consumed neither an acknowledgement nor an outcome before terminating the
  worker; every member remains operationally ambiguous because host evidence
  cannot establish whether the reader, supervisor, or executor received it.

Both cases performed no implicit retry and were followed by a separately
identified successful recovery request. A complete outbound write is not an
admission acknowledgement, just as a worker-local finish is not a transferred
outcome. Active signalling while one batch member is executing requires an
explicit member-execution state machine because the synchronous aggregate
prototype cannot otherwise process that control frame.

That state-machine experiment now exists privately. A five-member batch runs
members 0 and 1 synchronously, starts member 2 on the existing single execution
lane with the first-charge barrier, returns control to the supervisor, and
accepts the ordinary correlated cancellation frame. Member 2 returns exact
`FXCT0001 / cancelled`; only then do members 3 and 4 execute successfully. The
worker subsequently completes an ordinary recovery transform.

This proves active cancellation does not cancel or skip siblings and does not
require concurrent transformations. The asynchronous target is a supervision
mechanism: at most one transform executes at a time. The aggregate response
remains unchanged. The experiment does not select this private operation as a
protocol contract or prove natural cancellation races without the deterministic
first-charge barrier.

A separate natural-race operation removes the barrier and emits `started`
immediately after dispatch. Twenty-five five-member trials used a 20,000-item
source and cancellation delays of 0, 1, and 5 milliseconds. The observed target
outcomes were 20 exact cancellations and 5 successful completions, with zero
third dispositions, zero sibling failures, and successful worker reuse. The
late completions are valid: once completion is committed before the supervisor
processes the signal, completion wins. The experiment therefore preserves the
same race rule as the single-invocation lane without making cancellation
artificially deterministic.

The ASP.NET comparison executes the ordinary command and batch sizes 1, 8, 32,
and 128. Every result is compared exactly. Three endpoint calls rotate all five
lanes by one position to reduce fixed-order bias. Request identities are
prepared before timing. The largest tier has 1,000 members, giving eight
transactions at batch 128; this remains weak tail evidence but is materially
better than the two-transaction exploratory probe.

Command:

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -TargetFramework net10.0 `
  -IsolatedBatchBenchmark `
  -TieredRequests 1000 `
  -MeasurementRuns 3
```

Median throughput across three rotated reports:

| Tier      | Ordinary |  Batch 1 |  Batch 8 |  Batch 32 | Batch 128 | Batch-128 / ordinary |
| --------- | -------: | -------: | -------: | --------: | --------: | -------------------: |
| 5 items   | 25,281/s | 26,675/s | 76,840/s | 110,051/s | 131,995/s |                5.22x |
| 50 items  | 25,039/s | 24,401/s | 66,158/s |  84,631/s |  92,343/s |                3.69x |
| 500 items | 14,137/s | 13,495/s | 22,168/s |  25,541/s |  28,410/s |                2.01x |

Batch 1 is close enough to the ordinary command to remain a useful framing
oracle; its direction changes across tiers and runs. Every multi-member size
materially improves the median in every tier. The diminishing gain as execution
grows independently corroborates transport amortization rather than an engine
semantic optimization.

## Latency and allocation tradeoff

The response is emitted only after every member completes. Consequently each
member observes the entire transaction latency. Median per-run transaction
percentiles were:

| Tier / batch |        p50 |        p95 |
| ------------ | ---------: | ---------: |
| 5 / 8        |    94.1 us |   146.7 us |
| 5 / 32       |   272.5 us |   390.9 us |
| 5 / 128      |   946.5 us | 1,156.7 us |
| 50 / 8       |   112.3 us |   169.2 us |
| 50 / 32      |   362.6 us |   504.5 us |
| 50 / 128     | 1,348.5 us | 1,657.7 us |
| 500 / 8      |   336.9 us |   493.1 us |
| 500 / 32     | 1,202.0 us | 1,497.0 us |
| 500 / 128    | 4,360.4 us | 4,886.9 us |

The throughput gain therefore buys longer time to first result and a larger
worker-loss ambiguity set. Batch 128 is not an inferred default. Batch 8 already
captures a substantial part of the opportunity with a much smaller latency and
failure-radius increase.

Median managed allocation fell from about 2.8 KiB per ordinary member to about
1.7 KiB at batch 128. Two isolated samples reported implausibly large managed
allocation while their neighboring lanes did not. `GC.GetTotalAllocatedBytes`
is process-wide and includes unrelated ASP.NET activity, so those outliers make
the allocation column directional evidence only. Rust-side allocation is not
included.

## Disposition

AR-0021 should continue. The initial safe, sequential, input-order prototype
shows a large and repeatable throughput opportunity without changing engine
semantics. It is not ready for retention as a supported mode because:

- active member cancellation and broader composed-budget framing are absent;
- only one focused semantic-failure sibling test exists;
- aggregate-response latency and worker-loss ambiguity grow with batch size;
- CPU, worker memory, exact request/result pressure, and result-heavy behavior
  are not yet in the report;
- the private operation is not a stabilized or negotiated protocol version;
  and
- no engine-owned batching/admission rule has been selected.

The next experiment should add exact transport-pressure observations and
partial-completion fault injection before comparing aggregate with incremental
input-order outcomes. Completion-order delivery and intra-worker concurrency
remain unadmitted.
