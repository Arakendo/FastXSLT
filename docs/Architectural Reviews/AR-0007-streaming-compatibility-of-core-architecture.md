# AR-0007: Streaming Compatibility of Core Architecture

| Field | Value |
| --- | --- |
| Status | Deferred |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | XDM access, compilation metadata, execution strategy, buffering, and resource lifetime |
| Trigger | Preserve future streaming optionality without committing to implementation or conformance |
| Related ADRs | ADR-0001, ADR-0002 |
| Related evidence | AR-0001, AR-0003, and `docs/Evidence/peer-streaming-architecture-review-monday-2026-08-25.md` |

## Architectural question

Which assumptions in XDM access, compiled representation, runtime execution,
resource lifetime, and buffering would make future streaming prohibitively
invasive, and which low-cost boundaries should the initial tree evaluator
preserve without designing a streaming engine now?

## Terminology and claim boundary

**Streaming-compatible architecture** means the first engine has not spread
unnecessary full-tree and random-access assumptions into every semantic layer.
It preserves an evidence-led path to examine another physical or execution
strategy later.

**XSLT streaming conformance** is a standards feature involving the selected
edition's formal streamability rules and required behavior. FastXSLT does not
claim it. ADR-0007 deliberately excludes it from the initial profile; dedicated
suites and implementation evidence would still be required after renewed review.

Parsing XML as events, yielding output incrementally, or buffering a subtree is
not by itself XSLT streaming conformance.

## Trigger and evidence

The SDD already separates XDM meaning from physical XML representation, syntax
from semantic normalization, compilation from runtime, semantic results from
serialization, and admitted resources from host I/O. Those seams make future
streaming less invasive if the first implementation preserves them.

Many XPath and XSLT operations can require ancestors, reverse axes, document
order, repeated global navigation, keys, or access to data no longer available
from a forward-only source. A later engine might reject non-streamable cases,
materialize selected subtrees, build a full tree, or combine strategies. The
current project has no standards decision, evaluator, representative workload,
or profile showing which option is valuable.

The peer review recommends architectural optionality with no streaming
implementation. It also warns that a generalized abstraction introduced before
a second strategy exists may impose complexity and runtime cost without
preserving the right seam.

## Ownership and constraints

- XDM owns node identity, document order, names, values, and relationships
  required by the admitted profile, independent of physical storage.
- A physical source representation owns how it satisfies navigation, retention,
  and materialization needs. Tree-specific handles must not escape as semantic
  identity.
- XPath and XSLT own the semantic capabilities an expression or instruction
  requires; they do not own arena layout or event buffering.
- Compilation is the future owner of any static capability or streamability
  analysis metadata. Such metadata must retain source provenance.
- Runtime owns execution-strategy choice and per-invocation buffers. Another
  strategy must execute the same accepted semantics rather than become a second
  semantic engine.
- Resource snapshots own immutable admitted resource bytes and logical identity,
  not a requirement to pre-parse every batch source.
- Hosts own budgets and cancellation. Any subtree or document materialization
  consumes explicit memory budget and may not spill to disk under ADR-0002.
- Diagnostics must explain unsupported strategy requirements, limit exhaustion,
  or fallback where those become public behavior; silent semantic fallback is
  forbidden.

## Illustrative capability progression

```text
forward-only access
    -> bounded subtree materialization
    -> retained ancestor and context access
    -> arbitrary navigation and repeated access
    -> full-document materialization
```

This progression is not a type design or claim that all language constructs fit
one linear lattice. The M1/M2 implementation should record the capabilities
actually demanded by real expressions before naming abstractions.

## Alternatives

### A. Let every layer depend directly on one document arena

This is simple for the first evaluator and may be fastest initially. It makes
physical handles, random access, and full lifetime retention easy to spread into
semantic compilation, diagnostics, and public APIs, turning another strategy
into a broad rewrite.

### B. Design a universal navigation/provider trait system now

This appears future-proof but has no second implementation or measured caller.
It risks object-safety, lifetime, monomorphization, allocation, and semantic
leakage problems while abstracting the wrong operations.

### C. Build a concrete tree first with capability-aware ownership seams

Use direct private types inside the tree/XDM implementation, expose only the
semantic operations real callers require, record random-access assumptions, and
keep compilation meaning distinct from execution strategy. Introduce a trait or
other indirection only when another implementation or measured seam justifies
it.

### D. Implement tree and streaming execution together

This would test the boundary immediately but multiplies scope before the
standards profile and reference semantics exist. It is incompatible with the
first vertical-slice strategy and would make parity failures hard to attribute.

### E. Commit to tree-only execution permanently

This avoids abstraction and is a legitimate eventual product decision if
workloads support it. No current consumer evidence justifies closing the option.

## Findings and uncertainties

- Alternative C best matches ADR-0001: concrete first, with named ownership and
  no speculative public interface.
- A fully materialized tree is permitted and expected for the initial evaluator.
- Sealed in-memory resources do not require all sources to remain parsed as
  trees for the snapshot lifetime.
- Compilation needs an ownership seam for future requirement metadata, but no
  metadata schema or analysis pass is justified yet.
- Selective materialization can fit ADR-0002 only when memory use is explicit,
  bounded, diagnosed, and never silently spilled.
- Multiple execution strategies would require parity against one semantic
  reference and must not duplicate compilation meaning.
- The abstraction cost, useful workload set, standards requirements, hybrid
  fallback behavior, and achievable memory savings are unknown.

The first owned-tree experiment adds concrete capability evidence without
changing this disposition. The golden source uses document-root and child
navigation, expanded element names, parent relation, identity, semantic document
order, descendant text aggregation, and provenance. The builder owns its nodes
after source bytes are released and assigns order through semantic traversal.
It does not yet justify ancestors, reverse axes, keys, arbitrary repeated
navigation, typed values, namespace nodes, or a provider abstraction.

The completed private transform sharpens that inventory: `greeting/name`
performs two forward child-name steps from the document node, and `xsl:value-of`
uses the selected element's descendant text string value. Literal result
construction needs no source reverse navigation. This one path still provides
no evidence for a generalized provider, streaming strategy, or permanent
full-tree requirement.

The exact-name template-dispatch slice adds ordered selection of repeated
element children, expanded-name rule lookup, and reuse of each selected node as
the dynamic context for a separately compiled template body. The implemented
case is forward-only and consumes results in source document order, but the
reference evaluator still uses stable XDM node identity and a materialized
tree. This is additional navigation evidence, not a streaming claim or a reason
to introduce a provider abstraction.

Default apply-template selection adds recursive forward child traversal through
built-in document and element rules, while the built-in text rule consumes the
selected text node's owned value. Context-item `.` needs identity without
navigation. The current implementation is naturally forward-oriented for this
fixture, but template-rule lookup and stable dynamic context remain semantic
requirements independent of any future physical source strategy.

## Disposition

**Deferred. Do not implement streaming in the initial profile.** Preserve the
semantic/physical representation boundary, keep tree-specific access inside its
owner, allow compiled forms to gain requirement metadata later, and avoid
making batch snapshots imply eager full-tree retention. No current profile or
measured workload justifies a streaming strategy or generalized provider
interface. Reopen only when a named trigger supplies that pressure.

This disposition creates no streaming API, execution strategy, or conformance
claim.

## Required follow-up

- [x] ADR-0007 selects XSLT 3.0 as the reference edition and deliberately
  excludes any XSLT streaming-conformance claim from the initial profile.
- [ ] During M1/M2, inventory the navigation, order, ancestor, repeated-access,
  retention, and materialization needs of each implemented expression and
  instruction.
- [ ] Keep the first tree representation private and audit crossings where
  physical node or arena types leave XDM-owned code.
- [ ] Confirm a sealed batch can own source bytes without eagerly parsing every
  source into a retained tree.
- [ ] Add representative stream-friendly, random-access-requiring, and bounded
  materialization candidate fixtures without marking them supported.
- [ ] Prototype requirement metadata privately only after real cases reveal a
  useful distinction.
- [ ] Before adding indirection, benchmark the concrete tree path and the
  proposed abstraction on correctness-gated workloads.
- [ ] If another strategy is prototyped, differential-test it against the tree
  reference and measure peak memory, allocation, latency, and diagnostics.
- [ ] Define buffer budgets, cancellation, missing-capability behavior, and
  whether fallback is explicit before proposing public hybrid execution.
- [ ] Propose an ADR only when representative workloads and at least one
  alternative experiment justify a stable boundary or a deliberate tree-only
  decision.

## Reopening triggers

After disposition, reopen or supersede this review when the standards profile is
revised to include formal streaming, a consumer supplies large forward-processable workloads, tree memory
is a measured bottleneck, a concrete node API blocks another strategy, selective
materialization proves useful, or abstraction cost harms the tree evaluator.

## Review history

- 2026-08-25 -- Opened as Incubating from Monday's streaming-architecture peer
  review; implementation and conformance remain deferred.
- 2026-08-25 -- Recorded the first concrete navigation inventory from the owned
  golden-source tree experiment; no generalized interface was introduced.
- 2026-08-25 -- Executed the first child-path expression and recorded its exact
  navigation/string-value requirements; streaming remains unimplemented.
- 2026-08-25 -- Deferred implementation and abstraction work after the concrete
  tree slice confirmed the required seam can remain private; profile selection,
  representative workload pressure, or measured tree-memory pressure must
  trigger renewed review.
- 2026-08-26 -- ADR-0007 selected modern reference semantics while deliberately
  excluding streaming conformance from the initial profile. Architectural
  optionality remains deferred under this review's existing triggers.
