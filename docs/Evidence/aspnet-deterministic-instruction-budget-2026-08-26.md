# ASP.NET Deterministic Instruction-Budget Exhaustion

Date: 2026-08-26

## Question

Does an invocation-local semantic work budget cross the isolated ASP.NET worker
boundary as a structured limit failure, remain distinct from cancellation and
hard termination, and leave retained compiled/prepared state reusable?

## Method

The private worker protocol added one narrow experimental command carrying an
unsigned XSLT-instruction maximum for a single transform. The engine copies its
normal workbench limits for that invocation and replaces only the instruction
maximum. Ordinary and controlled-transform commands are unchanged.

The probe executed pinned XSLT30 `for-004` with a zero-instruction maximum,
asserted the returned diagnostic, then executed the ordinary workload on the
same worker process.

Command:

```powershell
./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 100 -MeasurementRuns 1
```

## Observation

- The limited request returned `FXCT0002 / limit`.
- Request identity remained `instruction-budget-exhausted`.
- Detail remained `xslt-instruction work budget exhausted: limit 0, consumed
  0, next charge 1`.
- The request was not retried and the worker was not terminated or replaced.
- A later ordinary request on the same process returned the expected
  `for-004` result.
- A focused direct Rust test asserts the same diagnostic and reuse behavior.

## Interpretation

This supplies host-boundary evidence for AR-0010's first guarantee class:
semantic work can stop through a deterministic engine-owned counter while the
worker remains healthy. It is visibly different from cooperative cancellation
and from supervisor-owned process termination.

The probe covers one work domain and one deliberately tiny budget. Direct Rust
tests already exhaust all implemented domains, but this does not yet prove
representative default selection, accounting overhead, adversarial wall-clock
bounds, a public limit configuration shape, or safe reuse after panic.
