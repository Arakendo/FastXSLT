# ASP.NET Active Cooperative Cancellation

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Host | ASP.NET Core targeting .NET 8 on Windows |
| Engine path | Supervised persistent isolated `fastxslt-worker` process |
| Workload | Pinned XSLT30 `for-004` over 500 deterministic items |
| Command | `./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 1000 -MeasurementRuns 1` |
| Claim | Private correlated active-signal evidence; not a natural latency, deadline, or public protocol claim |

## Protocol experiment

The worker now separates length-prefixed command reading from one active
execution thread. The supervisor owns the immutable engine, active request
identity, cooperative cancellation handle, and all response writes. The reader
may admit a cancellation command while execution is active, but the worker
still permits only one active transform. This is control-plane multiplexing,
not concurrent semantic execution inside one worker.

The managed client starts a controlled invocation and retains its operation
slot until the correlated completion arrives. Cancellation is a separate framed
command. The supervisor applies it only when its logical identity exactly
matches the active invocation. A deliberately unrelated identity left the
invocation pending; the following matching signal cancelled it.

## Deterministic charge-point probe

The native 500-item transform normally completes before a cross-process cancel
can reliably win the race. An initial uncontrolled attempt therefore returned a
valid result under the declared completion-wins rule. For deterministic
evidence, the workbench cancellation state optionally pauses the execution
thread at its first real engine-owned charge point. The worker acknowledges
that observed state, then the host sends the matching cancellation. This probe
path is private and absent from ordinary transforms.

The recorded run returned:

| Field | Observation |
| --- | --- |
| Code/category | `FXCT0001 / cancelled` |
| Request identity | `active-cooperative-cancelled` |
| Charge detail | `host cancellation observed while charging xslt-instruction work` |
| Signal-to-response | 0.5392 ms; an earlier successful barrier run observed 1.2906 ms |
| Process replacement | None |
| Unrelated signal | Ignored |
| Later same-worker result | `<out>500.00</out>` |

The timing includes managed framing, pipe transport, worker dispatch, barrier
release, engine failure projection, and response transport. Because execution
was deliberately paused, it is not a natural cancellation-observation latency,
an upper bound, or a deadline measurement.

Ordinary and pre-dispatch transforms remain synchronous on the worker reference
path. Only explicitly controlled invocations create the execution thread needed
for active control; the mechanism therefore does not impose per-invocation
thread creation on the previously measured warm path. The final smoke observed
18,359 ordinary transforms/second over 1,000 requests, but one run is a
regression check rather than a revised benchmark baseline.

## Race and guarantee boundaries

- A matching signal processed before semantic completion is cooperatively
  observed at a later charge point.
- A completion already committed by the supervisor wins; cancellation does not
  retroactively discard it.
- An unrelated or late signal does not cancel another logical request.
- Cooperative cancellation leaves the worker and immutable prepared generation
  reusable in this case.
- A non-cooperating call that cannot reach a charge point still requires the
  separately classified hard process-termination path.
- No timeout, cancellation frequency, natural wall-clock bound, retry policy,
  multi-tenant containment, or public managed API follows from this experiment.

## Disposition

AR-0010 remains Incubating and AR-0002 remains Proposed. The experiment removes
the protocol impossibility that blocked active signals, but representative
unpaused latency, cancellation/completion race sampling, broader diagnostic
parity, shutdown races, and an idiomatic ASP.NET `CancellationToken` adapter
remain open.
