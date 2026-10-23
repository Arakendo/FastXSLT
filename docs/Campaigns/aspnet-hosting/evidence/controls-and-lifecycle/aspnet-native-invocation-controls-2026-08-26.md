# ASP.NET Native Scalar Invocation Controls

| Field    | Value                                                                                                       |
| -------- | ----------------------------------------------------------------------------------------------------------- |
| Date     | 2026-08-26                                                                                                  |
| Boundary | ADR-0008 native workbench ABI extended by ADR-0009                                                          |
| Workload | Pinned XSLT30 `for-004` compiled and prepared once                                                          |
| Host     | ASP.NET Core targeting .NET 8                                                                               |
| Command  | `./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 100 -MeasurementRuns 1` |
| Claim    | Private diagnostic and lifecycle evidence; not a public cancellation or limit API                           |

## Method

The managed wrapper called one synchronous controlled-transform export with a
copied logical request identity, a scalar cancellation flag, and a scalar
maximum XSLT-instruction count. The native layer constructed the same safe Rust
`WorkbenchCancellation` and invocation limit used by the direct reference
path. No callback, foreign control handle, borrowed memory, asynchronous native
completion, or same-handle concurrent invocation was used.

The live ASP.NET operational probe executed, in order, an invalid-identity
failure, ordinary recovery, pre-signalled cancellation, zero-instruction budget
exhaustion, another ordinary recovery on the same engine handle, malformed
source initialization, two independent concurrent handles, disposal checks,
and the existing isolated operational experiments.

## Results

| Case                         | Code       | Category    | Request identity              | Exact detail                                                                 |
| ---------------------------- | ---------- | ----------- | ----------------------------- | ---------------------------------------------------------------------------- |
| Pre-dispatch cancellation    | `FXCT0001` | `cancelled` | `native-controlled-cancelled` | `host cancellation observed while charging xslt-instruction work`            |
| Zero XSLT-instruction budget | `FXCT0002` | `limit`     | `native-instruction-budget`   | `xslt-instruction work budget exhausted: limit 0, consumed 0, next charge 1` |

The ordinary recovery after both controlled failures returned the exact
`for-004` result:

```xml
<?xml version="1.0" encoding="UTF-8"?><out>36.02</out>
```

Focused native tests also rejected cancellation flag `2` as
`FXFFI0009 / boundary` and recovered on the same retained handle. Rust tests
asserted every field of the binary failure envelope rather than matching
display prose in the managed adapter.

The extension added no unsafe block and reused the already-reviewed immediate
request-identity copy. The audited surface remains two unsafe blocks and now
contains ten exported symbols with twelve scoped unsafe-code allowances, as
accepted by ADR-0009 and enforced by `scripts/verify.ps1`.

## Interpretation and limits

This establishes native parity for two invocation-local outcomes already
proven through direct and isolated execution. Neither failure poisons compiled
or prepared state.

It does not establish active mid-execution cancellation. The cancellation flag
is known and signalled before native execution begins, so there is no race,
deadline, or response-latency claim. In-process execution still cannot promise
hard termination. Adding an active control handle or callback would introduce
new threading, lifetime, and disposal invariants and remains outside ADR-0009.

Generation replacement, unsupported-stylesheet diagnostic parity, public
managed exception mapping, defaults, and production pool policy remain open in
AR-0002.
