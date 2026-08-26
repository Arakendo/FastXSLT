# AR-0005: Semantic Inspection and Explainability Boundary

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Read-only inspection of resources, compilation, and execution |
| Trigger | Embedded hosts need diagnosis without private Rust types or human-output scraping |
| Related ADRs | ADR-0001, ADR-0002 |
| Related evidence | AR-0002, AR-0003, `docs/Evidence/peer-database-documentation-review-2026-08-25.md`; future M2 caller fixture |

## Architectural question

What read-only semantic information should FastXSLT expose so a host can
explain admitted resources, compiled stylesheet requirements, diagnostics, and
bounded execution without stabilizing private parser, XDM, IR, optimizer,
arena, or cache representation?

## Trigger and evidence

Long-running applications will compile stylesheets outside request hot paths,
retain generations of sealed resource snapshots, and execute repeated
transformations. Operators and developers will need to answer which resource
was compiled, which dependencies were admitted, which profile and capabilities
are required, why compilation failed, which budgets were consumed, and whether
a reusable artifact can serve a request.

Human debug output is insufficient for ASP.NET tooling, compatibility reports,
test harnesses, and future UI or service adapters. Tosumu shows the value of a
structured inspection boundary that separates reportable findings from
inability to inspect. Its database commands and payload fields do not transfer
to FastXSLT.

FastXSLT now has private compiled stylesheet, resource snapshot, and runtime
experiments, but no public facade. There is still no evidence for stable fields,
schema versions, cost, or whether inspection should ultimately be a method,
immutable report, event stream, visitor, or adapter concern.

## Ownership and constraints

- Resource snapshots own logical identity, admitted-byte metadata, generation,
  limits, and qualified lookup observations selected by AR-0003.
- Compilation owns stylesheet dependencies, static profile requirements,
  static diagnostics, and semantic properties needed for safe reuse.
- Runtime owns per-invocation capability use, budget observations, messages,
  diagnostics, and result classification.
- Observability may provide events but cannot become an alternate owner of
  semantic truth.
- The facade owns a host-neutral read-only projection; adapters own JSON,
  managed objects, UI presentation, logging, and transport.
- Private trees, storage layouts, arena indices, optimizer IR, cache keys,
  allocation addresses, and module layouts are not semantic contracts.
- Inspection must not resolve resources, reopen files, execute extensions,
  mutate observable state, or widen host authority.
- Reports need bounded size and explicit treatment of source text, paths,
  parameters, and other sensitive data.

## Alternatives

### A. No inspection API; use diagnostics and logs

This minimizes public surface, but hosts cannot query successful compiled state
or explain reuse, dependencies, and capability needs. Logs are presentation,
not a request/response contract.

### B. Expose `Debug` or serialize private structures

Easy during development, but it stabilizes implementation accidents, discloses
sensitive state, and lets consumers depend on internals FastXSLT must replace.

### C. Immutable semantic inspection snapshots

The engine projects bounded host-neutral reports containing only admitted
semantic concepts, identities, requirements, classifications, and measurements.
This preserves implementation freedom but needs consumer evidence for fields
and compatibility.

### D. Event stream only

Events explain live work and may avoid large reports, but a late-joining host
cannot reconstruct all retained compiled state. Events may complement rather
than replace snapshots.

### E. Host-specific inspection surfaces

Each adapter can expose its platform's needs, but meanings may diverge. Adapters
should translate a common semantic projection rather than rediscover internals.

## Findings and uncertainties

- A successful compiled artifact needs inspectable semantic identity and
  requirements independently of diagnostic events.
- Inspection should project meaning and observations, never mutable internal
  handles or representation-dependent identifiers.
- Static compiled inspection, per-invocation summaries, and live tracing have
  different lifecycles and may require separate contracts.
- Report generation must be semantically inert and bounded.
- Real ASP.NET, conformance, and operator consumers have not yet identified the
  fields they must handle programmatically.
- Schema, Rust ownership, lazy/eager construction, compatibility, redaction,
  serialization, and cost remain unsupported by evidence.
- A private bounded owned projection now reports caller-supplied stylesheet
  identity, declared version, output semantics, template counts, instruction
  count, and implemented semantic feature counts. It survives its compiled
  owner and exposes no private tree, instruction body, match name, path,
  location, source text, or storage identity.

## Disposition

**Incubating.** M2 should build one private semantic inspection snapshot for an
implemented compiled stylesheet and exercise it through a consumer-shaped
caller. It must not expose private trees or claim a stable serialized schema.
Public stabilization waits for consumer use and coordination with AR-0004 and
AR-0006.

## Required follow-up

- [ ] Implement enough of M1/M2 to have a real resource snapshot, compiled
  stylesheet, dependency, standards profile, and diagnostic.
- [ ] Record the questions an ASP.NET workbench and conformance harness need to
  answer without reading human text.
- [ ] Prototype a bounded immutable projection containing no parser, arena, XDM
  storage, IR, optimizer, or cache implementation types.
  - [x] Project the implemented compiled slice into owned bounded semantic
    counts/settings without exposing representation types.
- [ ] Prove inspection performs no resolver calls, file access, extension
  execution, or semantic mutation.
  - [x] Exercise the private projection over an in-memory compiled fixture and
    prove the compiled program remains equal before and after inspection.
- [ ] Define sensitive fields, default redaction, source-text inclusion, report
  limits, and behavior after the inspected owner is dropped.
  - [x] Exclude source/literal/path/match/location data, bound copied text and
    feature kinds, and prove the owned projection survives its program.
- [ ] Distinguish static inspection, dynamic summaries, and tracing; measure
  their costs separately.
- [ ] Exercise unknown fields or versions through a non-Rust candidate if a
  serialized shape is proposed.
- [ ] If evidence converges, propose an ADR naming fields, ownership, bounds,
  authority, compatibility, and adapter responsibilities.

## Reopening triggers

After disposition, reopen or supersede this review when a host needs additional
stable meaning, an optimizer cannot evolve without breaking the projection,
reports expose sensitive data, report construction becomes material, or live
tracing is asked to replace semantic inspection.

## Review history

- 2026-08-25 -- Opened as Incubating from deferred findings in the Tosumu
  documentation review.
- 2026-08-25 -- Added the first private compiled-semantic projection. Its owned
  bounded fields answer implemented-slice questions without creating a public
  type or serialized schema; consumer fields and lifecycle remain Incubating.
