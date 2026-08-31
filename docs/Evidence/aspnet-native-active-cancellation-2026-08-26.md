# ASP.NET Native Active Cooperative Cancellation

| Field              | Value                                                                                                       |
| ------------------ | ----------------------------------------------------------------------------------------------------------- |
| Date               | 2026-08-26                                                                                                  |
| Host               | ASP.NET Core targeting .NET 8 on Windows                                                                    |
| Boundary           | ADR-0010 Rust-owned numeric native control handles                                                          |
| Workload           | Pinned XSLT30 `for-004` over 20,000 deterministic items                                                     |
| Deterministic runs | Three independent live host processes                                                                       |
| Natural runs       | Two unpaused samples of 25 managed-token races                                                              |
| Command            | `./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 100 -MeasurementRuns 1` |
| Claim              | Private cooperative-control evidence; not a deadline or hard-termination guarantee                          |

## Mechanism

The native layer owns cloneable Rust cancellation state behind a numeric
control handle. The synchronous transform clones that state under a registry
lock and performs no semantic work while any registry is locked. Cancellation,
first-charge observation, and release use scalar handle calls; no callback,
managed pointer, foreign borrow, or allocator crosses the ABI.

The managed adapter retains the control in `SafeHandle`, registers an ASP.NET
`CancellationToken` callback that invokes the scalar cancel operation, and runs
the blocking P/Invoke on a task. The ordinary synchronous transform continues
to use its original export and pays none of this task/control setup.

Release removes future signalling authority but does not invalidate a token
clone already owned by an invocation. The adapter disposes its cancellation
registration before releasing the control and retains the handle through
transform completion.

## Deterministic active signal

The workbench-only first-charge barrier paused execution after a real engine
charge was reached. Signalling an unrelated control did not complete or cancel
the target invocation. Signalling the target produced:

| Field            | Value                                                             |
| ---------------- | ----------------------------------------------------------------- |
| Code             | `FXCT0001`                                                        |
| Category         | `cancelled`                                                       |
| Request identity | `native-active-cancelled`                                         |
| Detail           | `host cancellation observed while charging xslt-instruction work` |

Three local signal-to-response observations were approximately 0.49 ms,
0.13 ms, and 0.13 ms. The barrier makes these attribution checks, not natural
cancellation-latency claims. The same compiled/prepared engine subsequently
returned the exact 20,000-item result. Repeated managed control disposal was
idempotent; native tests also proved cancellation after release fails.

## Natural managed-token races

Two unpaused 25-trial samples used the ordinary managed `CancellationToken`
adapter with no first-charge barrier. All 50 trials returned structured
cancellation, so each sample conserved `cancellations + completions = 25`.
Completion remains a valid race outcome even though it was not observed for
this input size.

| Sample |   Minimum |    Median |   Maximum | Outcomes                  |
| ------ | --------: | --------: | --------: | ------------------------- |
| 1      | 0.0258 ms | 0.0314 ms | 0.4316 ms | 25 cancelled, 0 completed |
| 2      | 0.0311 ms | 0.0415 ms | 0.4912 ms | 25 cancelled, 0 completed |

Natural cancellation was observed while charging both `xslt-instruction` and
`xpath-node-visit` work. Therefore code, category, request identity, and the
explicit charge-domain form of detail are stable, while one specific work
domain is not frozen for an unpaused race. An ordinary 20,000-item transform
succeeded afterward on the same engine.

## Safety and interpretation

The extension added no unsafe block. The audited surface remains two unsafe
blocks and now contains fifteen exported symbols with seventeen scoped
unsafe-code allowances, enforced by `scripts/verify.ps1`.

This closes active cooperative-cancellation parity for the current native
workbench lifecycle. It does not make in-process execution killable. A native
operation that stops reaching charge points cannot be reclaimed without ending
the ASP.NET process; the isolated worker remains the hard-containment mode.
Task scheduling cost, cancellation under sustained concurrent load, public
managed exception mapping, and representative consumer deadlines remain open.
