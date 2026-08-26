# AR-0002: ASP.NET Host Integration Boundary

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Non-Rust embedding, deployment, and performance boundary |
| Trigger | ASP.NET applications are a motivating FastXSLT consumer class |
| Related ADRs | ADR-0001 |
| Related evidence | Future ASP.NET consumer workbench, AR-0003, AR-0010, and end-to-end benchmarks |

## Architectural question

How should an ASP.NET application compile, retain, invoke, cancel, observe, and
deploy FastXSLT while preserving engine semantics and achieving useful
end-to-end performance?

## Trigger and evidence

FastXSLT is intended to be used by other applications rather than primarily as
a standalone executable. ASP.NET is a concrete example where compiled
stylesheet reuse and high request throughput matter.

No representative ASP.NET caller, transform family, deployment target, latency
budget, throughput target, payload distribution, or concurrency profile has yet
been measured. Rust-core speed therefore cannot select the host mechanism.

## Ownership and constraints

FastXSLT owns XML/XDM/XPath/XSLT semantics, compilation, execution, structured
diagnostics, and engine resource accounting. The managed adapter owns idiomatic
.NET types, disposal, cancellation translation, deployment ergonomics, and safe
lifetime handling. The ASP.NET application owns stylesheet caching policy,
request authorization, application budgets, logging presentation, and service
failure policy.

An adapter must not create alternate transformation semantics or silently widen
filesystem, network, entity, or extension authority.

ADR-0002 requires managed/native or other host adapters to finish file import
and release handles before exposing a sealed snapshot to request execution.
Per-request transforms must not reopen source paths or create hidden temporary
artifacts.

## Alternatives

### A. In-process native library with a narrow stable ABI

Potentially lowest steady-state call overhead and direct deployment, with
substantial requirements for ABI design, ownership, panic containment, native
artifact packaging, architecture-specific distribution, and safe managed
wrappers.

### B. Out-of-process worker or sidecar

Provides failure and authority isolation and language-neutral integration, at
the cost of transport, serialization, lifecycle, and operational overhead.

### C. WebAssembly or component boundary

May improve portability and sandboxing while imposing runtime availability,
memory transfer, resource integration, and performance constraints that must be
measured in the actual .NET host.

### D. Generated or host-native executable artifacts

Could move repeated evaluation work into a host-friendly artifact, but risks a
second semantic backend and requires strict parity, provenance, deployment, and
security contracts.

## Findings and uncertainties

- Compiled stylesheet reuse is a required capability independent of host
  mechanism.
- Bounded resource snapshots and transform sets may move file loading, parsing,
  and compilation outside the per-request path; AR-0003 owns those semantics.
- Consumer-visible performance includes boundary conversion and deployment
  behavior, not only Rust execution time.
- Cancellation, resolver callbacks, diagnostic transfer, output ownership, and
  concurrent invocation are likely to discriminate among alternatives.
- AR-0010 distinguishes cooperative in-process supervision from hard process
  isolation. Any host-facing hardened-mode claim must identify which boundary
  is actually deployed and include its transport and lifecycle costs.
- Current evidence is insufficient to select a mechanism.

## Disposition

**Proposed.** Keep the Rust engine host-neutral and do not stabilize an interop
ABI or managed API until a bounded ASP.NET workbench compares viable mechanisms.

## Required follow-up

- [ ] Record representative transforms, input/output sizes, request concurrency,
  deployment targets, and latency/throughput budgets.
- [ ] Define a minimal host-neutral compile/transform lifecycle to exercise.
- [ ] Exercise snapshot creation/replacement and a batch of transforms without
  transferring identical resource bytes on every invocation.
- [ ] Verify imported files can be replaced during service operation while old
  in-flight requests continue on their sealed snapshots without held handles.
- [ ] Prototype at least the leading in-process and isolated alternatives.
- [ ] Measure cold start, compile-once/warm execution, marshaling, cancellation,
  errors, result transfer, and steady-state concurrency end to end.
- [ ] Accept an ADR defining the selected boundary and its safety invariants.

## Reopening triggers

After disposition, reopen or supersede this review when a new host platform,
deployment constraint, isolation requirement, or measured boundary bottleneck
invalidates the selected mechanism.

## Review history

- 2026-08-25 -- Opened as Proposed after ASP.NET was identified as a motivating
  consumer class.
