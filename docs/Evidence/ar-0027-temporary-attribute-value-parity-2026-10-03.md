# Temporary attribute value parity

Date: 2026-10-03. Scope: close the remaining ordinary `@key` value-path blocker
following the private temporary for-each dispatch repair.

The unchanged AR-0027 regression is now enabled and passing. Growth, pre-sized
and frozen documents produce `<out><v>1/2:alpha</v><v>2/2:beta</v>|1</out>` with
identical output and all-domain execution/serialization charges. Candidate
source owners can be dropped before results are serialized. This closes one
bounded correctness seam; it does not select a capacity policy or establish
general temporary-tree XPath parity.

## Implementation boundary

LocationPath classifies a plain relative attribute step only when origin,
single-step shape and all predicate fields prove eligibility. A private
[attribute projection](../../crates/fastxslt/src/runtime/value_evaluator/temporary_attribute_path.rs)
reads attributes from the invocation-owned temporary focus. Named attributes
match expanded names, not lexical prefixes; unqualified names never match a
namespaced attribute. Attributes are selected completely before string values
are read, preserving traversal-before-value failure ordering. Each attribute
visit and each selected string value is charged. Existing result append logic
charges text bytes before retaining the borrowed attribute payload.

The shared value evaluator calls this projection for ordinary location-path
value-of expressions under temporary focus. It retains the typed first-node
versus separator-joined distinction without changing compilation rules. General
paths and predicates return `FXRT0007 / unsupported` with expression provenance,
rather than manufacturing a source focus. Other temporary scalar families and
sorting are not admitted by this change.

No prepared mutation, new resource authority, public DOM, provider trait,
alternate execution backend, cross-invocation sharing or unsafe code is added.
The typed temporary tree remains invocation-owned under ADR-0017.

## Differential and control evidence

Five source/temporary pairs compare exact serialized values for `@key`, two
namespace-qualified `key` attributes with distinct URIs, `@*`, and `@missing`.
Every pair includes an attribute-free second element. Source and copied
temporary attributes agree, including separators and empty output behavior.
The existing compiler still rejects `@p:*`; no executable namespace-wildcard
support is inferred from the internal name-test representation.

Control tests exercise exact and one-less limits for XPath visits, string-value
visits and result text bytes. Deterministic cancellation on the last attribute
scan charge yields exact `FXCT0001` with the XPath-visit domain; fresh execution
then returns `<out>alpha;beta</out>`. Predicate rejection retains expression
provenance. An instruction-level version override remains explicitly unsupported
at compilation; this tranche does not claim to resolve compatibility contexts.

The restored AR-0027 fixture independently covers copied attributes, temporary
focus size/position, source parent-path deduplication, capacity candidates,
exact work-domain parity and result/source lifetime separation.

```text
cargo test -p fastxslt --all-features temporary_attribute_paths -- --nocapture
cargo test -p fastxslt --all-features capacity_execution_preserves_temporary_focus_paths_and_result_retirement -- --nocapture
```

## Cohesion review and remaining work

Full verification passes with 1,534 core tests and 47 ignored probes. Formatting,
strict Clippy, workspace tests, documentation, local Markdown links, conformance
inventory/source integrity and unsafe-surface checks pass. The official
`wasm32-unknown-unknown` build check passes, with filesystem hard-link fallback
warnings. These differential tests were not executed in a WASM runtime.

The new projection is a 74-line private child under the existing value-evaluator
subject, not another flat runtime file. Its inputs are a compiled eligible path,
temporary focus, result sink and invocation control; it owns neither compilation,
variable storage, source acquisition nor serialization. The value evaluator
remains a pressured 3,116-line composition owner; the 2,957-line XPath path owner
gains only a classifier beside its existing origin/predicate invariants. Retain
these parents for this bounded repair and revisit decomposition at the next
broader scalar/path checkpoint. This does not discharge their decomposition debt
or establish build-time or throughput improvements.

Production capacity growth and default DTD denial remain unchanged. OASIS
coverage remains 2,485 / 3,173 (78.32%); these hand-authored controls add no corpus
credit. AR-0027 remains Incubating. Broader temporary operations, relationship
capacity, checkpoint publication semantics and host-boundary measurements remain
open before capacity adoption.
