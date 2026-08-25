# Peer Project Review: Tokimu and Weaver XSLT

| Field | Value |
| --- | --- |
| Reviewed | 2026-08-25 |
| Peer root | `F:\LocalSource\tokimu` |
| XSLT peer | `F:\LocalSource\tokimu\third-party\weaver-xslt` |
| Purpose | Inform the initial FastXSLT work environment |

## Material reviewed

The review covered Tokimu's root contributor instructions, Cargo workspace,
documentation index, SDD and testing guidance, ADRs, Architectural Reviews,
plans, notes, checkpoints, change requests, scripts, editor settings, and CI
shape. It also covered the embedded Weaver XSLT contributor instructions,
project layout, specifications index, accepted ADRs, AR index, plans, evidence,
corpus guidance, golden tests, conformance layout, and package scripts.

This was a structural and governance review, not an independent audit of either
engine's correctness or conformance.

## Useful patterns adopted

### Document authority is encoded by location

Tokimu distinguishes current design, binding ADRs, evidence-led ARs, executable
plans, working notes, and history. Weaver applies the same model more compactly
to an XSLT product. FastXSLT adopts that distinction and includes placement and
lifecycle guidance in `docs/README.md`.

### ARs and ADRs have different jobs

Tokimu's AR process retains the question, evidence, alternatives, findings,
disposition, follow-up, reopening triggers, and append-only history. ADRs retain
only accepted binding decisions and consequences. FastXSLT adopts both templates
and opens AR-0001 instead of pretending the standards target is already known.

### Corpus evidence pressures architecture

Tokimu treats examples as architecture-driving evidence. Weaver combines small
golden transforms, focused unit tests, upstream conformance suites, and backend
parity. FastXSLT adopts a repository-level golden corpus and a tiered testing
strategy, while explicitly separating selected-case success from conformance.

### Semantic ownership stays in the engine

Weaver's ADR-0001 keeps XML parsing replaceable while owning XDM, XPath, XSLT,
and execution semantics. Tokimu likewise distinguishes replaceable mechanics
from project-owned meaning. FastXSLT records the same boundary as current design
direction without selecting Weaver's JavaScript DOM dependency or physical node
representation.

### Diagnostics and host policy are cross-cutting contracts

Weaver makes structured diagnostics a public product concern, and both projects
make URI/resource boundaries explicit. FastXSLT carries source-located structured
diagnostics and deny-by-default ambient resource access into its SDD and agent
guidance before implementation begins.

### Repeatable gates belong at the repository root

Tokimu standardizes format, Clippy, tests, and docs expectations. Weaver adds
typecheck, lint, packaging, and strict documentation builds appropriate to its
TypeScript/site stack. FastXSLT adopts the Rust gates in a PowerShell script and
CI workflow; packaging and documentation-site tooling are deferred until those
products exist.

## Patterns deliberately adapted or deferred

### Workspace breadth

Tokimu's many crates and corpus packages reflect an established multi-domain
engine. Copying that shape into a new XSLT project would create stable boundaries
without callers. FastXSLT starts with one crate and private logical layers under
ADR-0001.

### Weaver's product decisions

Weaver targets XSLT 3.0/XPath 3.1, a DOM wrapper, JSON-serializable IR, an
interpreter plus native TypeScript/JavaScript backend, readable generated code,
and ESM packaging. These are evidence that the boundaries can work, not defaults
for Rust FastXSLT. The standards target, XDM representation, IR, execution
strategy, and generated artifacts remain open.

### Large documentation taxonomies

Tokimu has conversations, lessons, checkpoints, change requests, dependency
audits, libraries, and many campaign-specific plan/evidence trees. FastXSLT adds
only categories with an immediate owner and routing rule. Checkpoints, change
requests, archives, dependency audits, and a docs site can be added when actual
workflow pressure exists.

### External suites and vendoring

The peer repositories carry or reference substantial third-party fixtures.
FastXSLT creates only corpus policy and a first-party seed case. No external
suite should enter the repository before its license, provenance, integrity,
versioning, and selection process are documented.

## Resulting decisions and open questions

- ADR-0001 accepts an evidence-led modular monolith.
- AR-0001 keeps the initial XSLT/XPath standards profile under review.
- The SDD defines logical ownership, explicit host authority, structured
  diagnostics, and verification principles without inventing a public API.
- Parser choice, data representation, IR, execution model, CLI, bindings,
  streaming, schema awareness, and unsafe optimization remain non-decisions.

## Limitations

The peer repositories were reviewed from local working trees, not immutable
release tags. Their files may contain work newer than their public releases.
No performance results, suite counts, architecture claims, or dependency safety
claims were independently reproduced for this review.

