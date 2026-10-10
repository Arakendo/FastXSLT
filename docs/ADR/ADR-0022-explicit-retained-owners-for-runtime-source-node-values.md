# ADR-0022: Explicit Retained Owners for Runtime Source Node Values

- Status: Accepted
- Date: 2026-10-07
- Opened: 2026-10-07
- Related reviews: [AR-0028](../Architectural%20Reviews/AR-0028-owner-qualified-runtime-node-values.md)
- Related ADRs: ADR-0012, ADR-0013, ADR-0017, ADR-0021
- Supersedes: None

## Context

Unchanged OASIS CD04 cases `mdocs14`, `mdocs16` and `select59` require ordinary
node values to preserve their document owner through variables, focus switches
and mixed selections. Current local-ID values rely on the active source and
cannot safely be reinterpreted against another document. Existing unsupported
guards prevent that reinterpretation. The checkpoint remains 2,493 / 3,173 exact
comparisons (78.57%); acceptance earns no additional feature or corpus credit.

AR-0028's test-only explicit-owner reference proves colliding-ID separation,
effective-view identity, owner lifetime, sealed resolution reuse, COW and
complete-clone frame parity, live document admission, failure cleanup and
concurrent snapshot/frame isolation. It delegates semantics to the existing
qualified-node carrier rather than introducing another executor.

The invocation-table reference reduces document-owner clone operations for
large selections but does not reduce selection-buffer size. Complete allocation
scopes show higher total and peak requested bytes in every measured cell.
Local release probes show cheaper large-vector clone/drop and slower carrier
lookup. Its borrowed selections also prevent mutable admission while values
remain live. These are private reference observations, not compiled or host
performance evidence.

## Decision

Use explicit immutable document-owner retention for admitted runtime ordinary
source-node values, with semantic identity qualified by document origin and
local node identity. Do not introduce an invocation owner-table/token indirection
as the initial production representation.

This ADR selects the ownership direction, not a public node type, an exact
Rust field layout or completed executable support. Compiled consumers remain
unsupported until their integration gates
pass independently.

### Identity and lifetime

Document-origin identity and effective navigation ownership are distinct. An
engine-assigned document-origin identity qualifies ordinary node identity
together with the local node ID. Effective views derived from that document
preserve this identity, while independently prepared or reparsed documents do
not share it even when logical resource identity and bytes are equal. The
retained effective owner determines navigation and visible relationships; it
is not itself an additional node-identity component. Logical resource identity,
URI spelling, resolver-cache membership and allocation address are not
document-origin identity.

- Each retained value identifies its actual owning effective document. Focus
  changes cannot reinterpret an existing local ID against the new source.
- Shared owner handles retain immutable state; their allocation address is not
  semantic identity. Equal-byte reparses remain distinct documents. Repeated
  sealed-resource resolution retains the invocation's existing document owner.
- A selection may retain one owner when single-owner eligibility is proven;
  mixed selections must qualify each member. Grouping cannot change authored
  sequence order, node identity or the effective owner used for navigation.
- Runtime values, views, selections and resolver state remain invocation-owned.
  Lexical frames may share value-kind maps only under ADR-0017, with safe COW
  mutation, cross-kind shadowing and the complete-clone oracle preserved.
- Owners remain valid through all referencing frame lifetimes. Releasing the
  final value does not promise immediate document reclamation if the invocation
  resolver cache or another legitimate owner still retains it. Invocation
  teardown must release all invocation-owned references.
- Preparing another admitted document while values are live must not invalidate
  those values, freeze ordinary resolver growth or require reparsing old owners.

### Effective relationships and shared consumers

Apply the invocation's whitespace policy before a source-semantic selection can
retain or expose its nodes. Retain the effective owner, not merely an origin
token or raw prepared arena that can reintroduce hidden whitespace. ADR-0012's
complete derivation remains the differential oracle. Visible origin/local-ID
identity survives both representations; effective relationships need not match
the unstripped source.

XPath selection, focus, name/value/identity functions and navigation consume
the actual qualified node through the shared semantic machinery. Parameters,
globals and aliases must preserve the owner when their respective consumers
are admitted. No namespace-specific or multi-document alternate executor is
introduced. ADR-0021 namespace occurrences retain their existing derived
owner/element/prefix identity and do not acquire fabricated ordinary node IDs.

Within one document, preserve the existing qualified order and deduplication.
Cross-document identity tests do not select cross-document order. Operations
requiring general cross-document ranking remain explicitly unsupported until
that separate rule and its controls are admitted.

### Source context and caches

Retain fresh source-context key indexes and document-rooted match memberships
when the first compiled integration switches documents. The existing caches
implicitly depend on one effective source and compiled context: expanded key
names and template indexes do not independently qualify their local-ID payloads.
An owner-qualified node must not consult a cache built against a different
source merely because its local ID or origin token coincides.

Preserve ADR-0013's charged, bounded membership construction and uncached
fallback. No cross-owner, cross-view, cross-program or cross-invocation cache
reuse follows from this decision. A future cache extension requires its own
review of effective relationships, accounting, identity and lifecycle.

### Accounting and boundaries

Charge traversal and selection retention before corresponding retained growth;
observe cancellation at real work points. Partial selections must not become
visible on failure. Preserve request identity, expression/source provenance
and structured failure category across owner switches. Exact production charge
formulas must be pinned through each compiled operation; the test-reference
formula is not adopted implicitly.

Retaining an owner grants no resource authority. Resolution continues through
the explicit sealed snapshot and host denial rules, checked before cached reuse.
No ambient I/O, DTD expansion, retry, mutable document access or global cache is
introduced. Source-derived prepared storage and stylesheet-derived compiled
state remain immutable and free of invocation values.

Results remain semantically separate from source ownership and serialization.
Copy consumers must construct result-owned dynamic payload, preserving the
existing static-result namespace sharing exception. No source owner or carrier
becomes part of the public facade, native ABI or worker/WASM protocol.

## Consequences

Owner-aware values can survive document switching without bare-ID aliasing or
a mutable registry whose token lifetime must be coordinated with frames.
Explicit reference retention can increase clone/drop traffic and keep source
owners alive longer; COW sharing and proven single-owner grouping can avoid
unnecessary copies without changing this ownership direction.

The initial integration must preserve or adapt principal-source ownership
without silently deep-copying or reparsing prepared documents. Borrowed caller
lifetimes, shared prepared storage and effective-view retention must be proved
at that seam; this decision does not mandate a deep clone just to acquire an
owner handle. Measure complete adaptation cost and ordinary single-document
execution, not only the retained-value microbenchmark.

Implementation proceeds through private named owners under ADR-0004. Existing
local-ID fast paths may remain only where eligibility proves one effective
owner for their entire use. No representation change implies that every
expression or node kind is executable.

## Alternatives considered

An invocation table offers cheaper large-vector cloning, but adds lookup,
admission, table-qualified token validation, broader retention and a live-value
mutation problem. The current evidence does not justify that additional
lifecycle seam. Reconsider only after representative compiled pressure shows
explicit retention is materially costly.

Detached scalar views remain useful when consumers need only admitted scalar
semantics. They cannot replace owner-qualified nodes for navigation or identity.
Keeping explicit unsupported boundaries remains the fallback for all unproved
combinations.

## Non decisions

This decision does not select general cross-document ranking, namespace-valued
frames, temporary-node carrier migration, general `document()` grammar,
template execution during global constructor initialization, external DTD/entity
support, cache reuse across effective owners, public provider types, mutation,
cross-generation interning, a second backend or unsafe code. It does not claim
XSLT 1.0 or 3.0 conformance, a speedup or completed prototype API stability.

## Validation

Admit `mdocs14` as the first compiled vertical slice:
principal-source globals must retain their source during secondary-document
dispatch. Before crediting that slice:

- Prove actual globals, aliases, parameters and focus retain the correct owner
  wherever that slice uses them; keep unused combinations unsupported.
- Differentially compare effective views and complete derivation, COW frames
  and complete clones, and eligible cached/uncached match behavior.
- Pin colliding local IDs, repeated resolution, failed admission, exact budgets,
  active cancellation, diagnostics and final cleanup through compiled execution.
- Exercise concurrent compiled invocations and overlapping generations without
  retaining invocation values in prepared or compiled state.
- Prove key and match caches remain confined to their source/program context.
- Measure ordinary single-document latency, allocation, throughput and complete
  owner adaptation/retention cost before retaining the integrated carrier.
- Run unchanged `mdocs14`, then separately classify `mdocs16` and `select59`;
  conserve the full OASIS denominator rather than crediting anticipated support.
- Run workspace verification and official WASM checks, preserving adapter and
  unsafe surfaces. Keep explicit-owner and table references as test evidence.

Acceptance selects the architectural direction; these compiled gates
remain implementation obligations, not completed evidence.

## Acceptance history

- 2026-10-07 -- Accepted with the explicit distinction between document-origin
  node identity and effective navigation ownership. The maintainer requested
  no other architectural changes; compiled integration and corpus gates remain
  outstanding.
