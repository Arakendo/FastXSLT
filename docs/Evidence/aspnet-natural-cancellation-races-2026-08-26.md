# ASP.NET Natural Cancellation Races

| Field       | Value                                                                                                       |
| ----------- | ----------------------------------------------------------------------------------------------------------- |
| Date        | 2026-08-26                                                                                                  |
| Host        | ASP.NET Core targeting .NET 8 on Windows                                                                    |
| Engine path | Supervised persistent isolated `fastxslt-worker` process                                                    |
| Workload    | Pinned XSLT30 `for-004` over 20,000 deterministic items                                                     |
| Trials      | 25 unpaused started-transform/cancel pairs                                                                  |
| Command     | `./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 100 -MeasurementRuns 1` |
| Claim       | Workload-specific natural cancellation-race evidence; not a deadline or general latency bound               |

## Limit propagation prerequisite

The first attempt at a larger natural workload exposed that prepared input used
a private 1,024-event XML limit even though `WorkbenchLimits` explicitly
declared 100,000 events. The builder now receives its parser event and depth
limits from the workbench boundary. A focused Rust test admits and transforms a
600-item source that exceeds the former private ceiling. This makes the
workbench configuration effective; it does not select future product defaults.

The 20,000-item source is approximately 720 KiB, remains under the 1 MiB
resource bound, generates about 40,000 XML start/end events, and remains under
the configured XDM node and work limits.

## Method

Each trial:

1. started one controlled transform without the first-charge barrier;
2. received the worker's correlated start acknowledgement;
3. immediately sent a matching cancellation command;
4. awaited either a valid committed result or `FXCT0001 / cancelled`; and
5. conserved that outcome in the 25-trial denominator.

After all trials, the same process executed an uncancelled request and returned
`<out>20000.00</out>`. Controlled transforms alone used the execution-thread
seam; the ordinary recovery path remained synchronous.

## Result

| Outcome                 | Count |
| ----------------------- | ----: |
| Structured cancellation |    25 |
| Valid completion        |     0 |
| Other failure           |     0 |

Cancellation signal-to-response observations were:

| Statistic | Milliseconds |
| --------- | -----------: |
| Minimum   |       0.0952 |
| Median    |       0.1309 |
| Maximum   |       0.4285 |

These values include managed framing, pipe transport, worker event dispatch,
engine observation at a local charge point, structured projection, and response
transport. They are natural for this local workload in this run, but are not an
upper bound. Scheduling, system load, dependency calls, other stylesheet
shapes, output work, deployment topology, and cancellation timing can change
them.

The earlier 500-item unpaused attempt completed before cancellation arrived.
Together, the observations exercise both declared race outcomes: small work can
commit first; longer work can observe cancellation first.

## Disposition

The isolated boundary now has evidence for pre-dispatch cancellation,
deterministic charge-point routing, and unpaused completion-versus-cancellation
races. AR-0010 remains Incubating and AR-0002 remains Proposed. Representative
consumer transforms, adversarial dependency calls, sustained race sampling,
shutdown interaction, broader diagnostic parity, and an idiomatic managed
`CancellationToken` adapter remain open.
