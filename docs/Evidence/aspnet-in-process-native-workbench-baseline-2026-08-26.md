# ASP.NET In-Process Native Workbench Baseline

Date: 2026-08-26

## Scope

This is the first Windows x64 ASP.NET 8 observation of FastXSLT through the
experimental ADR-0008 native ABI. It is not a stable ABI, production package,
representative application benchmark, or general processor ranking.

## Boundary

The unpublished `fastxslt-dotnet-workbench` `cdylib` uses the same safe
`ExperimentalEngine` as the isolated worker. The ABI copies bounded source,
stylesheet, request-identity, result, and diagnostic bytes. Rust retains engines
and outcomes behind numeric handles; no Rust pointer or allocation ownership
crosses the boundary. The managed wrapper uses runtime P/Invoke marshalling and
`SafeHandle` disposal.

The only first-party unsafe operations are the ADR-0008 export attributes, one
validated input-slice/copy helper, and one validated output-copy helper. The
engine crate remains `unsafe_code = "forbid"`.

## Verification

The live workbench asserted:

- byte-exact `for-004` output through isolated and in-process lanes;
- `FXWB0003 / invalid` for an empty request identity;
- `FXXM0002 / invalid` for malformed source initialization;
- valid transformation after the invocation failure;
- concurrent execution through two independent native engine handles;
- idempotent managed double-dispose; and
- managed rejection of use after disposal.

Rust boundary tests additionally cover null/zero, null/nonzero, oversized input,
insufficient output capacity, result copying, unknown handles, consumed outcome
handles, and repeated release. A deliberate local panic test confirms permanent
quarantine state, but a full exported panic probe and managed-process quarantine
test remain outstanding.

The stable Windows MSVC toolchain reported that its Miri component is not
available, and sanitizer flags require a nightly toolchain. Neither Miri nor
AddressSanitizer was therefore run at this checkpoint; this is an explicit tool
coverage gap, not a passing result. The normal gate now counts exactly two
unsafe blocks, nine export attributes, and eleven scoped allowances, and rejects
unsafe Rust in every other first-party source file.

Command:

```powershell
./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 1000 -MeasurementRuns 3
```

## Initial warm observation

All paths compiled/prepared once and materialized each serialized result. The
FastXSLT lanes executed the exact pinned XSLT30 `for-004` source and stylesheet.

| Lane | Run 1 | Run 2 | Run 3 | Median |
| --- | ---: | ---: | ---: | ---: |
| FastXSLT isolated worker | 14,214/s | 16,809/s | 17,521/s | 16,809/s |
| FastXSLT in-process native | 266,042/s | 371,457/s | 321,926/s | 321,926/s |
| Microsoft in-process XSLT 1.0 equivalent | 90,084/s | 94,093/s | 106,207/s | 94,093/s |

The native/isolated ratio is about 19.15× by ratio of medians. The median of the
three per-run ratios is about 18.72×. The Microsoft lane executes an equivalent
XSLT 1.0 rewrite because it rejects the exact XPath 2.0 expression.

## Interpretation and limits

For this tiny warm transform, process framing and scheduling dominate enough
that removing them exposes a much faster engine path. The result establishes
that the in-process candidate is worth continued measurement and that the
ADR-0008 audit surface has produced material consumer-boundary evidence.

It does not establish the same ratio for larger transforms. The native wrapper
currently serializes calls on one engine handle; cancellation, generation
replacement, instruction-limit configuration, same-handle concurrency,
cold-load decomposition, p50/p95/p99, native allocation, whole-process memory,
Miri, sanitizer coverage, and exported panic quarantine remain open. The
isolated lane retains hard process-termination capabilities the native lane
cannot provide.
