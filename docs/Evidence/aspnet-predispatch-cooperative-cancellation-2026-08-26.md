# ASP.NET Pre-Dispatch Cooperative Cancellation

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Host | ASP.NET Core targeting .NET 8 on Windows |
| Engine path | Persistent isolated `fastxslt-worker` process |
| Stylesheet | Pinned XSLT30 `for-004` |
| Command | `./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 10 -MeasurementRuns 1` |
| Claim | Private pre-dispatch cancellation and diagnostic-parity evidence; not an active cancellation or deadline contract |

## Method and result

The host submitted `cooperative-cancelled` with an invocation cancellation state
already signalled. The isolated worker passed that state into the ordinary
engine execution path. FastXSLT observed it at the first engine-owned charge
point and returned:

| Field | Exact value |
| --- | --- |
| Code | `FXCT0001` |
| Category | `cancelled` |
| Request identity | `cooperative-cancelled` |
| Detail | `host cancellation observed while charging xslt-instruction work` |

The private direct Rust facade test asserts the same code, category, request
identity, and detail. The process identifier was unchanged after cancellation,
and the same compiled stylesheet and prepared input successfully executed
`cooperative-after-cancel`. No worker was killed or replaced.

This distinguishes a reportable cooperative cancellation from
`FXWB2001 / worker-terminated`, budget exhaustion, and transport failure. It is
also evidence that a cancellation observed before semantic mutation does not
poison the retained generation.

## Deliberate limitation

The current worker protocol serializes one request and one response on a single
stream. It cannot receive another control message while the execution loop is
inside a transform. The workbench therefore reports
`activeMidExecutionSignalSupported = false` and exposes no misleading managed
`CancellationToken` overload.

Active cancellation needs a multiplexed reader/executor boundary or separate
control channel, request-correlated signal ownership, cancellation/completion
race rules, and wall-clock observation measurements. Killing a worker remains
hard termination and is not a fallback implementation of cooperative
cancellation. Deadlines remain a separate operational guarantee class.

## Disposition

This closes only pre-dispatch cancellation transport and one exact
direct-versus-isolated diagnostic comparison. AR-0002 remains Proposed and
AR-0010 remains Incubating pending active signalling, cancellation races,
observation latency, broader diagnostic parity, and representative workloads.
