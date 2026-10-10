# XSLT 1.0 current predicate lexical boundaries

Date: 2026-10-04

## Result

The private XSLT 1.0 predicate adapter now distinguishes an unqualified
`current()` call from the same text inside a string literal or a longer
function name. An edge regression also exposed and repaired a UTF-8 boundary
panic in the shared predicate operator scanner. These are correctness repairs,
not a new multi-document or general expression capability.

## Reproduction and repair

Before repair, `part[@id='current()/@refid']` failed because textual
normalization replaced the quoted content with the private outer-context
marker. Both outer-attribute and outer-context normalization now search at
character boundaries, outside single- and double-quoted literals. A preceding
name character or colon prevents a suffix such as `mycurrent()` or
`p:current()` from being treated as the unqualified function.

Testing `part[@id=écurrent()/@refid]` exposed a separate panic in predicate
operator splitting: its byte loop sliced a Rust string inside `é`.
The scanner now iterates character boundaries while retaining quote and
parenthesis depth. This expression remains unsupported, but no longer panics.
No Unicode expression family is inferred from that repair.

## Executable evidence

`xslt10_current_predicate_normalization_preserves_quoted_text` checks both
quote forms, literal `current()` values, a quoted reference followed by a real
call in an `or` predicate, actual outer-attribute selection, source location,
and cancellation at a real node-visit charge. Literal-only paths also match
the ordinary XPath reference selector in node identity and charged visits.
Longer, qualified, and Unicode-prefixed function names remain unadmitted.
The ordinary XPath parser still does not acquire the XSLT function.

`xslt10_path_union_keeps_quoted_current_text_distinct_from_outer_focus` is a
hand-authored golden transform through sealed resources, compilation, union
selection, numeric sorting, and serialization. Two independent requests
produce exactly `<out>b|current()|current()/@refid|</out>`. This golden evidence
does not count as an OASIS pass.

## Corpus and deferred boundaries

The unchanged local OASIS CD04 archive remains identified by SHA-256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
Replay command:

```powershell
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'mdocs17'
```

Coverage remains 2,491 / 3,173 exact XML comparisons (78.51%), with 2,692
initialized cases, 2,643 successful executions, 478 initialization failures,
49 execution failures, 14 visible XML mismatches, four comparator gaps, and
423 / 431 expected-error credits. No expected result, overlay selection, or
comparison rule changed. Another 48 exact cases are required for 80%.

`mdocs14` still needs principal-source values to retain their owner while an
external document supplies template focus; the earlier
[global document evidence](oasis-xslt10-global-literal-document-variable-2026-09-30.md)
already identifies that seam. `mdocs17` still fails on a qualified stylesheet
path with an outer source attribute comparison. This tranche does not admit
cross-document `current()` evaluation, ownerless node values, namespace-valued
variables, external entities, or legacy namespace-copy recovery.

## Cohesion review

Under ADR-0004, retain `path_experiment.rs` at 2,991 lines for this bounded
lexical correction. The helper remains private to XSLT path normalization;
operator splitting remains owned by the shared predicate parser. Neither
introduces runtime state, resource policy, an alternate evaluator, or a public
API. The XPath directory remains at 49 direct files; no new sibling or
ceremonial subgroup is added. Tests stay in the existing path and path-union
owners. Reopen decomposition at the next broader grammar change, rather than
combining a semantic repair with an unrelated navigation rewrite.

## Validation

Focused regressions and the complete `scripts/verify.ps1` gate pass: formatting,
strict workspace Clippy, 1,585 core tests (57 manual probes ignored), 22 native
adapter tests (three ignored), four WASM adapter tests, 18 worker tests,
documentation, Markdown links, conformance provenance, and metadata inventory.
The official all-features `wasm32-unknown-unknown` adapter check and complete
OASIS replay also pass. Windows incremental-cache hard-link warnings fell back
to copying; they did not require weakening a verification gate.

No throughput or allocation improvement is claimed. This checkpoint remains
uncommitted alongside the preceding corpus work.

## Secondary document sequence classification

A subsequent 2026-10-04 frontier audit found that `mdocs14` was classified as
an invalid unbound variable even though its global `$html-template` had already
been materialized from sealed `document()` resources. General variable
sequence dispatch supports principal-source nodes and selected temporary
trees, not a secondary document carrying cross-document values. Calling this
binding unbound obscured the actual engine capability gap.

The existing `apply-templates` and `for-each` dispatch owners now identify a
bound literal-document value through the existing shadow-aware runtime lookup
and report `FXRT0007 / unsupported` at the consuming instruction. Neither
copies secondary node IDs into the principal arena nor widens supported
document-variable execution. Apply-template argument evaluation retains its
existing precedence before this rejection. No acquisition or DTD policy changes.

Two focused regressions cover both consumers, global and local document
bindings, consuming stylesheet provenance, request correlation, successful
local principal-source shadowing, and genuinely missing variables remaining
`FXRT0002 / invalid`. These hand-authored tests do not add corpus credit.

Reproduction:

```powershell
cargo test -p fastxslt --all-features secondary_document_sequence
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs14
```

The complete replay remains 2,492 / 3,173 exact XML comparisons (78.54%), with
2,693 initialized cases, 2,644 successful executions, 477 initialization
failures, 49 execution failures, 14 XML mismatches, four comparator gaps, and
423 / 431 expected-error credits. `mdocs14` moves from invalid `FXRT0002` to
unsupported `FXRT0007`; no case becomes a pass. The historical checkpoint above
precedes the shared substring tranche and is not rewritten.

Read-only investigation also confirmed that `TestIdFuncInComplexStruct`
selects the reference IDs and patterns but differs in leading whitespace,
and `Sorting__91753` has the reference collation order but differs in whitespace
inside its first empty `br`. Those observations do not establish a normative
engine defect or justify comparator relaxation. The legacy-byte BVT reference
files remain undecodable under their current metadata; their bytes are not
silently reinterpreted to gain credit. `Keys_PerfRepro2` still first fails at
dynamic numeric conversion inside `format-number`; its broader aggregate and
temporary-value expressions need shared semantics, not a report-specific rule.

Under ADR-0004, retain the existing 6,700-line runtime composition root for
this small dispatch guard, with its previously recorded decomposition debt.
The for-each owner remains 359 lines and its focused tests 408 lines. No new
source file, public type, runtime ownership, unsafe surface, or backend is
introduced. Further cross-document execution remains a separate ownership
decision, not an implication of the corrected diagnostic.

Final validation passes `scripts/verify.ps1`: formatting, strict workspace
Clippy, 1,601 core tests (57 manual probes ignored), 22 native tests (three
ignored), four WASM tests, 18 worker tests, documentation, local Markdown
links, conformance provenance, and metadata inventory. The official
all-features `wasm32-unknown-unknown` adapter check, complete release OASIS
replay, and diff checks also pass. Windows incremental hard-link warnings use
the compiler's copying fallback; no gate is relaxed. This checkpoint remains
uncommitted with the preceding work. No performance gain is claimed.

## Current attribute document lookup

The next 2026-10-04 slice closes unchanged `Lotus/mdocs_mdocs17#1`. Its
stylesheet-local month table is selected through `document('')` with a
terminal attribute equality against `current()/@sequence`. The earlier
same-document outer-focus marker could not represent that comparison honestly
inside another document.

The existing private literal-document value plan now optionally retains two
statically resolved expanded attribute names. At execution, the outer
attribute is converted while the active source still owns `current()` focus.
Only its optional string value crosses into the lookup; no source node ID,
variable frame or prepared-tree mutation crosses that boundary. The existing
charged location-path evaluator selects candidates in the sealed document.
Ordinary value output takes the first matching candidate in document order.
This is a bounded terminal `[@candidate=current()/@outer]` form, not a new
general expression interpreter or multi-document node carrier. Additional
predicates are rejected explicitly, preventing an earlier `current()` predicate
from being interpreted against the lookup document's focus.

Missing outer attributes do not compare equal to empty candidate attributes;
empty present attributes do. Unprefixed attribute tests have no namespace,
and prefixed tests resolve against the consuming stylesheet instruction.
Attribute visits and both string conversions use real invocation controls.
Temporary focus returns located unsupported behavior instead of borrowing
principal-source IDs. Modern grammar, general document-variable selection,
broader current-function composition, namespace-valued variables, external
entities, and live acquisition remain unadmitted.

Five hand-authored regressions cover stylesheet and external sealed lookups,
qualified attributes, empty and missing values, duplicate matches and
first-node conversion, complete-reference/visibility-view parity, concurrent
invocations sharing immutable source and compiled state, exact N/N-minus-one
node-visit and string-work limits, cancellation and recovery, denied lookup
resources, static unbound prefixes, modern rejection, and temporary-focus
rejection with provenance. These regressions are not additional corpus credit.

```powershell
cargo test -p fastxslt --all-features current_document_lookup
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs17
```

The complete release replay gains exactly one XML comparison and loses none:
2,493 / 3,173 (78.57%). Initialized cases increase to 2,694 and successful
executions to 2,645; initialization failures decrease to 476. Execution
failures remain 49, XML mismatches 14, comparator gaps four, and expected-error
credits 423 / 431. The archive hash and XSLT 1.0 edition recorded above remain
unchanged; no fixture bytes, comparison rules, selection or denominator change.
Another 46 exact cases are needed for 80%.

Under ADR-0004, retain the existing literal-document compiler and evaluator
owners for this bounded extension. The value compiler is 3,613 lines and the
value evaluator 3,230 lines; their helpers still own static lowering and
dynamic lexical execution respectively. They gain no host acquisition or
portable node-identity responsibility. A document-value compiler/evaluator
extraction is the named checkpoint before a broader document-expression
family. The seven-file golden-test directory remains below the density trigger;
the new test owner groups current-focus/document-ownership invariants rather
than adding tests to the large parent. Retention accounting includes both
expanded names. No unsafe surface, public API or alternate backend changes.

Final `scripts/verify.ps1` passes with 1,606 core tests (57 ignored), 22 native
tests (three ignored), four WASM tests and 18 worker tests, plus formatting,
strict Clippy, documentation, Markdown links and conformance provenance and
inventory gates. The official all-features WASM build-target check and complete
release replay pass. Work remains uncommitted; no performance gain is claimed.

## Literal document value decomposition

The named ADR-0004 checkpoint is now completed as a behavior-preserving
extraction. `literal_document_value_compiler.rs` owns static lowering of the
existing scalar literal-document path and terminal current-attribute filter.
`literal_document_value_evaluator.rs` owns outer-focus scalarization, sealed
lookup, effective-view navigation and first-match string conversion. The
parent compiler still selects compatibility grammar; the parent evaluator
still rejects temporary focus before dispatching this source-only operation.

The compiler child is 113 lines and its parent decreases from 3,613 to 3,513;
the evaluator child is 85 lines and its parent decreases from 3,230 to 3,157.
Dependencies remain explicit and private: compilation consumes static XML
names and path parsers, while evaluation consumes existing resource,
whitespace-view, path, string and work-control owners. Neither child owns
acquisition authority, a portable node carrier, global initialization,
runtime-variable storage, host policy or a public contract. No algorithm,
allocation strategy, error order, work charge or unsafe surface changes.

The instruction-compiler directory now contains 18 direct files and therefore
requires a density review. Retain the existing feature-named files at this
checkpoint: they share the instruction-compilation subject, and the extracted
owner remains a private child of value compilation rather than a new semantic
layer. Review a value-expression subdirectory before another expression-family
extraction; do not create a file-count bucket. The evaluator child directory
contains six files and remains below the density trigger.

All five focused lookup regressions pass after extraction. The complete
release replay conserves 2,493 exact XML comparisons, 2,694 initialized cases,
2,645 successful executions, 476 initialization failures, 49 execution
failures, 14 XML mismatches, four comparator gaps and 423 expected-error credits.
The official WASM target check passes. This checkpoint adds no conformance
credit and makes no performance claim.

Full `scripts/verify.ps1` also passes after extraction: formatting, strict
Clippy, workspace tests, documentation, Markdown links, unsafe-surface checks
and conformance inventory/provenance checks. The focused regressions remain
part of the 1,606 passing core tests. Documentation links and `git diff --check`
pass after this evidence and roadmap update. All work remains uncommitted.
