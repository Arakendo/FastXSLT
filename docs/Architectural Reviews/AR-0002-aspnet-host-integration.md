# AR-0002: ASP.NET Host Integration Boundary

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-26 |
| Scope | Non-Rust embedding, deployment, and performance boundary |
| Trigger | ASP.NET applications are a motivating FastXSLT consumer class |
| Related ADRs | ADR-0001, ADR-0008, ADR-0009, ADR-0010 |
| Related evidence | `docs/Evidence/aspnet-isolated-persistent-worker-baseline-2026-08-26.md`, `docs/Evidence/aspnet-xslt-engine-comparison-2026-08-26.md`, `docs/Evidence/aspnet-tiered-workload-and-bounded-concurrency-2026-08-26.md`, `docs/Evidence/aspnet-native-vs-isolated-tiered-comparison-2026-08-26.md`, `docs/Evidence/aspnet-native-invocation-controls-2026-08-26.md`, `docs/Evidence/aspnet-native-generation-and-diagnostic-parity-2026-08-26.md`, `docs/Evidence/aspnet-native-active-cancellation-2026-08-26.md`, `docs/Evidence/aspnet-worker-recovery-and-generation-replacement-2026-08-26.md`, `docs/Evidence/aspnet-predispatch-cooperative-cancellation-2026-08-26.md`, `docs/Evidence/aspnet-active-cooperative-cancellation-2026-08-26.md`, `docs/Evidence/aspnet-natural-cancellation-races-2026-08-26.md`, AR-0003, AR-0010, and future end-to-end comparisons |

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
- A first ASP.NET 8 workbench now imports one pinned source and stylesheet,
  closes the import handles, transfers their bytes once to a persistent isolated
  Rust worker, and reuses one compiled stylesheet and prepared source across
  correlated HTTP requests. The safe length-prefixed protocol has 1 MiB frame
  bounds and transfers structured engine failures.
- The first baseline deliberately serializes work through one worker. Three
  local 1,000-transform runs observed a native `for-004` rate of about
  15,336–27,305 transforms/second, but excludes meaningful concurrency,
  cancellation, worker restart,
  snapshot replacement, consumer workloads, and comparison with in-process
  interop. Current evidence therefore remains insufficient to select a mechanism.
- A five-run warm comparison placed the isolated FastXSLT path near an
  in-process SaxonCS-HE 13.0.0 path for the exact tiny XSLT 2.0 workload:
  23,994 versus 28,297 median transforms/second. Microsoft's in-process
  `XslCompiledTransform` measured 108,612 median transforms/second for an
  equivalent XSLT 1.0 rewrite, but could not execute the original XPath 2.0
  expression. These are useful comparison bounds, not representative product
  performance or an in-process FastXSLT boundary.
- SaxonCS remains a gitignored, locally acquired comparison dependency. Its
  13.0.0 NuGet payload carries license text inconsistent with Saxonica's public
  SaxonCS-HE licensing description, so FastXSLT does not distribute or restore
  it by default.
- A four-worker isolated pool preserved correlation and exact semantics across
  deterministic 5-, 50-, and 500-item tiers. Five-run median FastXSLT throughput
  ranged from 25,966 to 5,550 transforms/second sequentially and from 84,939 to
  22,903 transforms/second concurrently. The pool makes prepared-state memory
  multiplication explicit; cancellation and production restart/replacement
  policy remain open.
- A private two-worker fault probe acknowledged a deliberately non-cooperating
  request, terminated only its process, returned an explicit
  `worker-terminated` disposition without retry, initialized a replacement from
  the same sealed generation, and preserved sibling plus subsequent execution.
- A private host-owned generation experiment initialized a replacement before
  atomic promotion, routed new requests to its explicit generation identity,
  and allowed an acquired old-generation request to drain before disposal.
- A changed-resource variant imported and closed host file streams, renamed and
  removed both original source and stylesheet while the old generation remained
  leased, reused the host paths for new bytes, and observed distinct old/new
  results after promotion. Paths remained outside engine identity and authority.
- Pre-dispatch host cancellation now crosses the isolated boundary as
  cooperative engine state, preserves exact direct/isolated code, category,
  request identity, and detail, and leaves the worker reusable.
- A supervised reader/executor experiment now carries an identity-correlated
  signal after execution reaches a real charge point, ignores an unrelated
  identity, returns structured cancellation, and reuses the same worker. Its
  deterministic first-charge barrier prevents a natural latency claim.
- In 25 unpaused 20,000-item trials, cancellation won every race with
  0.0952–0.4285 ms signal-to-response and the worker remained reusable. An
  earlier 500-item trial completed first. These are local workload observations,
  not deadline bounds or managed API selection.
- A private `CancellationToken` overload now requests correlated cooperative
  cancellation while retaining structured engine failures and the
  completion-wins race rule. A four-case direct/isolated diagnostic matrix
  retained code, category, request identity, detail, and source identity/span.
- A narrow invocation-local instruction-budget command now returns structured
  limit exhaustion without retrying or replacing the worker, followed by a
  successful ordinary request on the same compiled/prepared process.
- The leading in-process design now has accepted ADR-0008: a
  workbench-only native library with copied buffers, numeric handles, no
  callbacks, exact unsafe containment, and permanent panic quarantine. No
  native implementation remains an unpublished workbench candidate.
- The first native candidate executes byte-exact `for-004`, preserves invalid
  identity and malformed-XML diagnostics, uses managed `SafeHandle` ownership,
  and runs independent handles concurrently. Three 1,000-call warm runs observed
  321,926/s native versus 16,809/s isolated by medians on this tiny workload.
- The native candidate now runs the same deterministic 5-, 50-, and 500-item
  tiers through four independent handles. Three-run native/isolated throughput
  ratios narrowed from 21.82x to 1.49x sequentially and from 14.71x to 1.50x
  concurrently as transform work increased. This demonstrates both a material
  tiny-call boundary cost and its workload-dependent amortization. Managed
  allocation excludes Rust in both paths, and whole-host native working set is
  not an attributable engine-memory measurement.
- ADR-0009 now admits one controlled native transform carrying only a validated
  cancellation flag and XSLT-instruction limit. The live managed probe retained
  exact `FXCT0001 / cancelled` and `FXCT0002 / limit` diagnostics and reused the
  same engine after both failures. This is pre-dispatch cooperative control,
  not active signalling, a deadline, or hard termination.
- A host-owned native generation experiment fully initialized a replacement
  engine pool before atomic promotion, routed new requests to its changed
  two-item prepared source, retained a leased old one-item result, and disposed
  the retired pool only after lease release. The native matrix also now retains
  the direct/isolated unsupported-stylesheet code, category, identity, and span.
- ADR-0010 adds Rust-owned numeric control handles without callbacks or another
  unsafe block. A deterministic post-charge signal retained exact `FXCT0001`
  fields, ignored an unrelated control, and recovered the same engine. Two
  unpaused 25-trial managed-token samples conserved all outcomes as cancellation
  and naturally observed both XSLT-instruction and XPath-node-visit domains.

## Disposition

**Proposed.** Keep the Rust engine host-neutral and do not stabilize an interop
ABI or managed API until a bounded ASP.NET workbench compares viable mechanisms.

## Required follow-up

- [ ] Record representative transforms, input/output sizes, request concurrency,
  deployment targets, and latency/throughput budgets.
- [x] Define a minimal experimental host-neutral compile/prepare/transform
  lifecycle to exercise without accepting it as the public API.
- [x] Exercise snapshot creation/replacement and a batch of transforms without
  transferring identical resource bytes on every invocation.
- [x] Verify imported files can be replaced during service operation while old
  in-flight requests continue on their sealed snapshots without held handles.
- [x] Prototype at least the leading in-process and isolated alternatives.
  - [x] Establish a persistent isolated-worker baseline with bounded frames,
    compile-once/prepared reuse, stable result correlation, and structured
    failure transfer.
  - [x] Accept ADR-0008 and establish the first in-process native candidate with
    copied buffers, numeric handles, structured outcomes, and managed disposal.
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
- 2026-08-26 -- Added the first ASP.NET 8 persistent isolated-worker baseline.
  It proves one-time resource transfer, compile/prepare reuse, correlation, and
  structured transport without selecting the production ABI or execution mode.
- 2026-08-26 -- Compared the warm exact workload with locally acquired SaxonCS
  and an equivalent XSLT 1.0 workload with Microsoft's built-in processor. Kept
  SaxonCS outside source control and left this review Proposed.
- 2026-08-26 -- Added tiered latency/allocation evidence and a bounded
  four-worker isolated pool without promoting the workbench protocol or pool to
  a supported host boundary.
- 2026-08-26 -- Added non-cooperating worker termination/replacement and explicit
  generation promotion/draining evidence. Kept restart policy and the workbench
  lifecycle private.
- 2026-08-26 -- Replaced imported source and stylesheet files while an old
  generation lease remained active, then proved old/new results stayed bound to
  their sealed generations.
- 2026-08-26 -- Carried an already-signalled cooperative cancellation through
  the isolated boundary with exact direct diagnostic parity and worker reuse.
  Active mid-execution signalling remains open.
- 2026-08-26 -- Added one-active-invocation control-plane multiplexing and a
  deterministic first-charge cancellation probe with correlation and reuse.
  Natural observation latency and managed API selection remain open.
- 2026-08-26 -- Sampled 25 unpaused cancellation races after propagating the
  explicit XML-event limit into prepared parsing. All larger-workload trials
  cancelled; the earlier small-workload completion preserved the opposite race.
- 2026-08-26 -- Added a managed-token adapter and matched four representative
  direct diagnostics across the isolated boundary. Public exception mapping and
  the in-process candidate remain unresolved.
- 2026-08-26 -- Carried deterministic XSLT-instruction budget exhaustion across
  the worker boundary and retained same-process reuse. The private command does
  not select a public limit-policy representation.
- 2026-08-26 -- Proposed the exact unsafe surface and safety contract for a
  workbench-only native .NET candidate. Retained the implementation gate under
  ADR-0003 pending explicit acceptance.
- 2026-08-26 -- Accepted ADR-0008 and executed the first native ASP.NET
  candidate. Exact output, two diagnostic phases, independent handles, disposal,
  and a three-run warm comparison passed; broader lifecycle parity remains open.
- 2026-08-26 -- Ran the same 5/50/500-item tiers through isolated workers and
  independent native handles. Recorded boundary-cost scaling, latency,
  concurrency, allocation scope, and retained-memory measurement limits without
  selecting a production default.
- 2026-08-26 -- Accepted ADR-0009 and carried pre-dispatch cancellation plus a
  deterministic instruction budget through scalar native ABI values. Exact
  diagnostic fields and same-handle recovery passed without adding an unsafe
  block; active native signalling remains open.
- 2026-08-26 -- Added managed native generation promotion/draining with changed
  prepared source semantics and completed the current representative native
  diagnostic matrix through unsupported-stylesheet parity. No ABI or unsafe
  surface changed.
- 2026-08-26 -- Accepted ADR-0010 and exercised active native cooperative
  cancellation through Rust-owned control handles. Deterministic and natural
  managed-token probes preserved diagnostics and reuse without claiming a
  deadline or in-process hard termination.
