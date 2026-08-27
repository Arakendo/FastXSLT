# ASP.NET Native Generation and Diagnostic Parity

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Host | ASP.NET Core targeting .NET 8 |
| Native boundary | ADR-0008/ADR-0009 version-zero workbench ABI |
| Stylesheet | Pinned XSLT30 `for-004`; focused unsupported `xsl:message` fixture |
| Command | `./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 100 -MeasurementRuns 1` |
| Claim | Private host-lifecycle and representative diagnostic-parity evidence |

## Representative diagnostic matrix

The live native probe now preserves all fields asserted by the existing direct
and isolated workbench matrix:

| Phase/outcome | Code | Category | Identity/provenance observation |
| --- | --- | --- | --- |
| Empty request identity | `FXWB0003` | `invalid` | No request identity is invented |
| Malformed prepared source | `FXXM0002` | `invalid` | Detail retains the supplied source identity |
| Unsupported `xsl:message` | `FXST1006` | `unsupported` | Detail retains stylesheet identity and exact `103..117` span |
| Pre-dispatch cancellation | `FXCT0001` | `cancelled` | Exact logical request identity and charge-point detail |
| XSLT-instruction exhaustion | `FXCT0002` | `limit` | Exact logical request identity and budget accounting detail |

The managed adapter selected behavior from the binary envelope fields rather
than parsing display strings. Ordinary execution on the same retained engine
succeeded after invocation-local cancellation and limit failures. Compilation
failures produced no engine handle.

## Native generation promotion

The host created `native-generation-001` from a one-item source, acquired an
explicit old-generation lease, and fully created `native-generation-002` from a
two-item source before entering the promotion lock. Promotion atomically changed
the current generation and retired the old one.

A new request returned:

```xml
<?xml version="1.0" encoding="UTF-8"?><out>2.00</out>
```

The old lease then executed against its original prepared source and returned:

```xml
<?xml version="1.0" encoding="UTF-8"?><out>1.00</out>
```

Releasing the old lease disposed its retired native pool; a subsequent call
through the retained test reference was rejected as disposed. New-generation
execution remained available. Generation identities were explicit host values,
not paths, hashes, native handles, or content-derived identities.

## Boundary consequences

Generation replacement is entirely managed host policy over independent native
engine pools. It adds no ABI operation and no unsafe Rust. Resource bytes cross
the native boundary once per engine in each candidate generation, before
promotion; transforms remain memory resident.

This closes representative diagnostic and generation-lifecycle parity between
the native and isolated candidates. It does not give in-process execution hard
failure containment. Active mid-execution native cancellation, public managed
types, production pool sizing/backoff, and representative consumer requirements
remain open in AR-0002.
