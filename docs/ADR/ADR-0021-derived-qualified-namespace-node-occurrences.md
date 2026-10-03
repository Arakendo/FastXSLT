# ADR-0021: Derived Qualified Namespace Node Occurrences

- Status: Accepted
- Date: 2026-10-01
- Opened: 2026-10-01
- Related reviews: [AR-0026](../Architectural%20Reviews/AR-0026-namespace-node-identity-and-navigation.md)
- Related ADRs: ADR-0004, ADR-0007, ADR-0012, ADR-0017, ADR-0018
- Supersedes: None

## Context

Fifteen unchanged OASIS CD04 scenarios first fail on namespace-node selection
or identity. Existing element bindings support fixup and counting, but ID-only
path results, focus, and variable values cannot carry a namespace node honestly.
Using its parent as focus would change name, value, identity, and copying.

AR-0026's safe test-only references demonstrate element-relative identity,
binding shadowing and undeclaration, implicit `xml`, whitespace-view identity,
temporary-owner qualification, bounded mixed ordering, scalar focus reuse,
parent navigation, and a narrow result-attachment subset. They do not establish
compiled namespace-axis execution or full runtime parity.

A generated 513-element / 17-binding probe retained 1,062,936 known bytes in
eager borrowed-occurrence vectors versus 2,048 in one derived selection. This
comparison excludes transient maps, prepared storage, and allocator overhead;
it is neither a full arena comparison nor evidence that derivation is faster.
It supports avoiding mandatory eager expansion, not a performance claim.

The standards reference is [XDM 3.1](https://www.w3.org/TR/xpath-datamodel-31/),
with legacy behavior selected by the existing compatibility profile. The
decision below chooses an integration boundary, not completed feature support.

## Decision

Derive namespace occurrences on demand over immutable source or temporary
trees. Carry them through the shared engine as private qualified semantic-node
values alongside ordinary tree nodes, without adding inherited namespace
occurrences to the prepared arena or assigning them fabricated arena IDs.

### Identity and lifetime

- A source occurrence is qualified by prepared document origin, owning element,
  and prefix. Effective whitespace navigation may differ without changing the
  identity of a visible occurrence.
- A temporary occurrence is qualified by invocation, temporary tree, owning
  element, and prefix. Lexical-frame clones preserve that identity; separately
  created trees and invocations do not alias when local numeric IDs coincide.
- URI strings, hashes, declaration locations, storage addresses of effective
  views or cloned temporary trees, and host paths are not namespace identity.
- The owner remains alive while a value can refer to it. Invocation-owned
  selections and views do not enter compiled/prepared state, global caches, or
  another invocation. Runtime handles and owner storage remain private.
- Selection retains available declaration provenance independently of the
  owning element. Missing temporary provenance remains absent; errors retain
  the available expression or construction site without inventing locations.

### Shared semantic consumers

XDM owns namespace identity and binding meaning. XPath owns qualified axis
results, normalization, predicates, and focus. Runtime temporary storage supplies
its declaration relationships without making XDM depend on runtime execution.
No public provider trait or alternate namespace evaluator is introduced.

The actual namespace node, rather than its parent element, supplies current-item
name/value, parent navigation, identity functions, and copy behavior. Namespace
string value and node-name namespace must remain distinct. Version/profile
rules remain explicit rather than modifying modern semantics to match archives.

Paths, mixed unions, focus, and variable bindings use a common qualified-node
contract for admitted combinations. Tree-only fast paths may retain local IDs
where eligibility is proven; conversion must preserve owner qualification.
Variable integration preserves ADR-0017's invocation-local COW isolation,
cross-kind shadowing, and complete-clone oracle.

Within one element, use stable lexical-prefix namespace order as the private
convention: Rust `str::cmp` ordering, case-sensitive and locale-independent,
without Unicode normalization. The empty/default prefix sorts first; implicit
`xml` participates normally rather than receiving a privileged position.
Preserve owning-element / namespace / attribute / descendant order.
Cross-document normalization must use a consistent owner-qualified order within
an execution, not payload equality or bare local IDs. Such combinations remain
explicitly unsupported until their ordering and lifecycle are tested; this ADR
does not select a document-ranking algorithm.

### Copying and control

A selected namespace copies into a distinct private namespace attachment item,
not an ordinary attribute or child. Result construction resolves attachment,
conflicts, timing, and namespace fixup before serialization. Dynamic copied
payloads are result-owned and safely outlive their source owner. ADR-0018's
immutable stylesheet-derived slices retain their existing separate ownership.

Charge declaration traversal, occurrence/sequence retention, conflict scans,
and copied payload bytes before the corresponding retained allocation. Observe
cooperative cancellation at real work points, bound collection growth, and
discard partial products on failure. Namespace URIs confer no acquisition
authority. Exact charge formulas are recorded and differentially verified per
admitted operation; this decision does not promise equal costs for different
algorithms or weaken existing invocation budgets.

Keep the complete safe binding, ordering, effective-document, frame-clone, and
result-copy references as applicable differential oracles. Admit syntax and
consumer combinations incrementally only after executable conservation tests.
Acceptance selects this direction; it does not turn unchecked features into
supported behavior or convert hand-authored controls into corpus passes.

## Non-decisions

This decision does not select a public node type, provider, facade, ABI, wire
format, pointer-based identity scheme, persistent cache, general namespace
constructor, schema profile, or unsafe exception. It does not admit concurrent
mutation, sharing invocation values across workers/generations, or general
cross-document ordering before its controls exist.

Legacy copy recovery, broader modern construction conflicts, template matching,
base-URI behavior, and atomization require profile-specific tests before their
respective operations are admitted. No behavior is inferred merely because a
qualified occurrence can be represented.

## Alternatives considered

**Eager namespace arena nodes** simplify ID-only consumers but impose inherited
binding retention on every prepared element. The current evidence does not
justify that mandatory cost. Reconsider if measured repeated traversal dominates
and a bounded alternative preserves prepared ownership.

**Namespace-specific execution or parent-element focus** avoids migration but
creates a second semantic path or returns wrong node meaning. Neither is admitted.

**Remain unsupported** is safe and remains the fallback for unproved
combinations, but leaves the shared consumer boundary unresolved.

## Consequences

Prepared representation and visible tree IDs remain intact. Runtime paths,
focus, variables, node functions, and result construction require deliberate
private integration. Derived lookup repeats traversal and may allocate bounded
temporary collections; it is not presumed faster. Checkpoint extraction follows
ADR-0004 rather than adding more responsibilities to pressured owners.

Two archival expected outputs assume `xml` is first. A different permitted
ordering may remain an exact-byte mismatch; keep that disposition visible
without changing fixtures or claiming semantic failure solely from ordering.
Current OASIS exact coverage remains 2,471 / 3,173 (77.88%).

## Validation

Before admitting each affected operation:

- Compare real source and temporary execution, current-item focus, parent
  navigation, names/values, identity, predicates, variables, dispatch, and copy
  against the appropriate complete references.
- Exercise shared whitespace views, complete derivation, COW mutation and
  deep clones, concurrent invocations, and overlapping generations without
  identity drift or retained invocation state.
- Prove bounded retention, cancellation, work exhaustion, failure cleanup, and
  honest source/construction diagnostics through actual compiled execution.
- Execute the unchanged fifteen-case OASIS frontier and pinned modern cases;
  conserve every disposition and separate permitted order from exact output.
- Measure namespace-heavy traversal/retention and ordinary tree-path latency,
  allocation, and throughput before retaining the integrated representation.
- Run workspace gates and official WASM-target checks without expanding the
  public or unsafe surfaces.

These integration gates are still pending. The existing test references establish
feasibility only. Reopen the decision if qualified values force a second executor,
break lifetime/accounting isolation, require eager prepared expansion, or cause
a material ordinary-path regression without a justified fallback.
