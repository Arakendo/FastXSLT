# OASIS XSLT 1.0 Computed Element Namespaces and Dispatch

The 2026-10-04 continuation fixes explicit computed-element namespace handling
under local XSLT 1.0 context. It gains two unchanged exact comparisons after a
behavior-preserving compiler extraction, bringing coverage to 2,490 / 3,173
(78.47%). A separate control exposed an unsupported temporary wildcard
selection reaching a panic; that path now returns a structured outcome.

## Scope and reproduction

The unchanged OASIS XSLT 1.0 CD04 archive has SHA256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
Run `scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Elements__83968'`
and the same command with `Microsoft/BVTs_bvt054` for visible comparisons.
The denominator remains 3,173. No fixture, expected bytes, comparator,
exclusion, production parser policy, acquisition authority, vendor revision,
public facade, wire representation, or unsafe surface changes.

## Compiler decomposition checkpoint

Following [ADR-0004](../../../../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md),
static and dynamic computed-element compilation move from the 4,102-line
instruction compiler into a private named child. Attribute namespace binding
retention remains with its existing callers; no host policy or runtime state
enters the extracted owner. Inputs are immutable stylesheet nodes and static
context; outputs are existing instruction variants and located compile errors.
Sequence traversal and shared expression services stay at the composition root.

The twelve focused computed-element tests pass at the extraction checkpoint.
The full unchanged replay conserves every counter, including 2,488 exact
comparisons, 2,689 initialized, and 2,640 successful executions. The extraction
and semantic repair therefore have separate observable checkpoints.

After repair the parent is 3,834 lines and the child is 321 lines. The child
owns QName interpretation, static namespace choice, admitted dynamic name
plans, and computed content composition. Dependencies remain private and
one-way; it owns no resolver, invocation values, public abstraction, or alternate
backend. Fifteen direct files in the instruction compiler directory trigger
a density review: retain the navigable instruction-feature owners together
for this checkpoint rather than create arbitrary buckets. Revisit when a
stable constructor or expression subsubject warrants grouping. The parent
remains pressured; this is a targeted responsibility extraction, not completed
decomposition. No build-time or runtime performance gain is claimed.

## Explicit namespace repair

[XSLT 1.0 section 7.1.2](https://www.w3.org/TR/1999/REC-xslt-19991116#creating-elements)
defines the expanded name using the supplied namespace and QName local part;
an empty namespace is null, and the lexical prefix need not be preserved.
The previous static and dynamic implementations rejected prefixed empty
overrides. Reserved-prefix rebinding could also serialize an illegal namespace
declaration; the new control reproduced that defect by reparsing its output.

The compatibility branch now discards prefixes for empty namespaces and
replaces reserved prefixes whose requested binding would be invalid. XML
namespace names use canonical `xml`; ordinary safe prefixes remain available.
Static and dynamic paths retain the same result-element representation.
Current-source and temporary-node names feed the shared runtime QName resolver
when this compatibility override applies. Private instruction flags preserve
local static version selection rather than consulting only the principal
stylesheet version. Inline flags participate in existing instruction-size
accounting; they retain no new heap owner.

Malformed QNames, reserved XMLNS namespace URIs, missing overrides, and the
existing modern behavior remain separate. This does not admit arbitrary AVTs
or a second runtime semantic engine.

Four focused compiled controls cover static, path-derived, variable-derived,
source-current and temporary-current names; null/ordinary/XML namespaces;
safe reserved-prefix replacement; local versus principal version context;
complete-reference versus visibility-view parity; concurrent reuse; exact and
one-less navigation/result-node budgets; cancellation; failure cleanup;
recovery; and construction provenance. The empty constructors charge result
nodes, not invented text bytes. All four originally failed against the prior
implementation, and the retained controls distinguish semantics from that
earlier failure.

## Temporary wildcard dispatch guard

The new temporary-focus control initially selected `$tree/*`. Compilation
produced `GlobalTemporaryChildren`, which the `for-each` dispatcher did not
handle before entering its tree-only source selector. That reached an
`unreachable!` branch rather than a reportable outcome.

The existing private `for_each` owner now returns `FXRT0007 / unsupported`
with request identity and instruction provenance for that selection. No new
wildcard algorithm is admitted. Named temporary paths remain executable and
provide the namespace parity control. A dedicated regression retains the
original wildcard counterexample for both local and global temporary bindings
and verifies reuse after failure. Eight focused `for_each` controls pass.

## Corpus accounting and validation

The semantic replay reports 2,691 initialized, 479 initialization failures,
2,642 successful executions, and 49 execution failures. Exact XML comparisons
increase from 2,488 to 2,490. Fourteen mismatches, four comparator gaps,
423 / 431 expected-error credits, and 134 direct-DTD attributions are unchanged.
The newly exact cases are `Microsoft/Elements__83968#1` and
`Microsoft/BVTs_bvt054#1`. The 80% checkpoint requires 2,539 exact comparisons,
another 49 cases; no initialization gain or controlled rejection substitutes
for an exact pass.

Workspace formatting, strict Clippy, tests, documentation, conformance
inventories, unsafe-surface checks, and local Markdown links pass. Core tests
report 1,573 passed and 57 ignored; native, WASM, and worker suites report
22, four, and eighteen passed respectively. Link verification checks 1,974
Markdown files. The official `wasm32-unknown-unknown` all-features check passes,
and the post-guard full replay preserves the counters above. No competitive
performance or broad XSLT 1.0 conformance claim follows from these two cases.
