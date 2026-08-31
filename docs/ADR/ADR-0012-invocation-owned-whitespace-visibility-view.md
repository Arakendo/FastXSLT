# ADR-0012: Invocation-Owned Whitespace Visibility View

- Status: Accepted
- Date: 2026-08-30
- Related reviews: AR-0007, AR-0009, AR-0013, AR-0016
- Related ADRs: ADR-0001, ADR-0002, ADR-0004, ADR-0007
- Related evidence: `docs/Evidence/ar-0016-source-access-inventory-and-safe-reference-2026-08-30.md`, `docs/Evidence/ar-0016-visibility-view-prototype-2026-08-30.md`, `docs/Evidence/peer-ar-0016-decision-readiness-monday-2026-08-30.md`, `docs/Evidence/ar-0016-decision-measurement-matrix-2026-08-30.md`, and pinned XSLT30 case `mode-1301`
- Supersedes: None

## Context

The admitted `xsl:strip-space elements="*"` case requires whitespace-only text
children to be absent from source semantics. XML parsing cannot apply that rule
because it is stylesheet-dependent. Reusable prepared XDM cannot be mutated or
prepared differently per stylesheet without violating source-derived identity,
concurrent reuse, and generation isolation.

A complete invocation-owned derived document provides a safe semantic
reference, but it clones all node payload and relationship storage. A private
visibility view instead shares immutable prepared node storage and retains only
the child-sequence overrides needed to hide strip-eligible text nodes.
Differential controls now cover every current `Document` accessor, XPath and
template traversal, containing string values, copying, focus position and
size, concurrent strip/preserve execution, and overlapping stylesheet
generations. The unchanged W3C `mode-1301` case passes through the view.

Across five source shapes, the view reduced total construction-plus-execution
time by 2.74x to 8.35x versus the complete clone and improved four-thread
throughput by 2.53x to 8.87x. On a 6,003-node source, allocator-requested peak
bytes were 32,408 for the view and 3,214,912 for the clone.

## Decision

For the exact admitted `xsl:strip-space elements="*"` policy, compose the
immutable prepared source with a private invocation-owned visibility view.

The view must:

- preserve each visible prepared node's `NodeId`, document order, expanded
  name, value, namespace information, source location, and provenance;
- hide strip-eligible whitespace text through the effective relationship seam
  before XPath, template selection, built-in rules, string values, copying,
  focus construction, or result construction observes the source;
- remain immutable after construction and local to one invocation;
- charge construction through the owning XDM work domain and observe the
  invocation's cancellation and budgets;
- share only immutable prepared node storage; and
- be dropped without mutating the prepared input, compiled stylesheet,
  snapshot, worker, or another generation.

Retain the complete safe derived-document implementation as a test-only
differential oracle wherever practical. A new source-semantic consumer must use
the effective `Document` access surface and receive a stripping parity control
before admission.

## Non-decisions

This ADR does not admit:

- general `xsl:strip-space` name tests or declaration precedence;
- `xsl:preserve-space`;
- `xml:space` or schema-aware typed whitespace semantics;
- a public or universal source-provider/view trait;
- a stylesheet-specific prepared-input variant;
- retention or reuse of a view across invocations or generations;
- a global or cross-snapshot cache;
- XSLT streaming conformance; or
- unsafe code.

Each broader semantic rule requires corpus pressure and focused review. A
retained or shared representation reopens AR-0009 and AR-0013 rather than
following from this decision.

## Consequences

Prepared input remains source-derived and reusable across preserving and
stripping stylesheets. The runtime owns composition of compiled whitespace
policy with invocation source state. Every current semantic consumer observes
one effective document without acquiring a generalized navigation abstraction.

Stripping has a measurable per-invocation cost relative to a preserving
stylesheet, but no cost is paid when the compiled policy is preserve. The view
is materially cheaper than cloning for all measured source shapes and does not
need retained reuse to break even against the reference candidate.

The physical view layout remains private and replaceable. The decision fixes
semantic ownership, identity, lifecycle, and access invariants—not a bitset,
map, vector, arena, cache key, or public Rust type.

## Validation

- Differentially compare the view with the complete reference at every current
  source accessor and full transform result.
- Execute unchanged pinned XSLT30 `mode-1301`.
- Prove effective child and descendant focus positions and sizes exclude hidden
  nodes before `position()` and `last()` are evaluated.
- Prove enclosing string values and source copying use effective relationships.
- Execute preserving and stripping stylesheets concurrently over one prepared
  source and overlap old/new stylesheet generations.
- Stop view construction through real cancellation and XDM budget charge
  points.
- Keep source locations and visible node identities equal to the prepared
  source.
- Run the normal FastXSLT verification suite.

Revisit this decision when a new source-semantic consumer bypasses the
effective access seam, a corpus case requires broader whitespace semantics, a
consumer requires effective-document inspection, a retained/shared view is
proposed, or representative measurement materially reverses the observed
construction, execution, memory, or concurrency result.
