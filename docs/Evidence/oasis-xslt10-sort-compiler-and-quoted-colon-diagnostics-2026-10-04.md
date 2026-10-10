# OASIS XSLT 1.0 Sort Compiler and Quoted Colon Diagnostics

The 2026-10-04 continuation separates sort compilation from the pressured
instruction compiler, then repairs an unsupported-versus-invalid diagnostic
gap. This is a conservation and classification checkpoint, not additional
conformance credit. The preceding exact checkpoint is 2,491 / 3,173 (78.51%).

## Extraction checkpoint

The private `sort_compiler` owns sort-key selection and typed data-type, order,
language, and case-order controls. It consumes immutable stylesheet XDM,
static-context helpers, and existing typed expression compilers, returning
typed sort plans and excluded sort-node IDs. Sequence-constructor traversal
remains in the instruction composition owner. The existing shared bounded
path helper remains there because computed element/attribute compilation also
uses it; those consumers do not acquire a dependency on sort-control lowering.

No runtime scope, source preparation, host policy, resource acquisition,
public API, retained representation, or alternate evaluator moves into the
new child. Calls remain direct safe Rust. No allocation or dispatch layer is
added to execution. Build-time or performance improvement is not claimed.

Before diagnostic repair, the extracted composition passes all workspace
verification gates, 24 sort-focused tests, and a full unchanged CD04 replay.
Every replay counter and the exact `sort36` failure remain unchanged. This
separates the mechanical move from the later classification change.

## Quoted colon repair

The unchanged `Lotus/sort_sort36#1` uses a valid composed sort expression with
the literal `Re: `. Its substring/boolean/numeric/concat composition is beyond
the current typed sort-key slice. Shared path validation incorrectly treated
the colon and whitespace inside that literal as malformed namespace syntax;
sort fallback also treated any colon as a reason to try QName parsing.

A shared private XPath lexical scan now identifies only unquoted colons for
these checks. It does not parse the composed expression, add sorting semantics,
or make every unsupported expression valid. A compiler regression covers
single- and double-quoted colons, bound qualified paths, unbound prefixes,
malformed QNames, legacy/modern context, and expression-site provenance.
String punctuation alone no longer manufactures an invalid namespace error.

The first full replay exposed a conservation dependency: forward-compatible
`system-property('xsl:version', 'foo:bar')` cases had relied on the same false
QName error for deferred failure. The static introspection owner now reports
the real wrong-arity error, `XPST0017`, and existing XSLT 1.x forward-compatible
processing defers that known expression error alongside syntax/type errors.
It does not defer unsupported capability. A compiled control verifies strict
legacy/modern rejection, dormant future-version deferral, provenance, quoted
commas, nested-call commas, and rejection of an overbroad whole-call match.
`function-available()` arity is deliberately not inferred from this check.

## Cohesion and density review

Under [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md),
retain the instruction composition owner after extracting the cohesive sort
family: 3,364 lines at extraction, 3,367 after the diagnostic repair. The child
has 550 physical lines including its focused
test. It depends on immutable semantic inputs and narrow typed compiler
helpers rather than a mutable context spanning the former owner's state.

The instruction compiler directory now has 16 direct files, triggering explicit
density review. Retain its named feature children at this checkpoint: sorting
adds one cohesive leaf, while moving existing value/boolean/constructor
families into a new hierarchy would combine this diagnostic repair with a
separate navigation campaign. Revisit grouping at the next coordinated
expression/constructor family extraction, not through numbered file buckets.

The shared path owner is 2,971 lines and its directory has 49 direct files.
This change adds one allocation-free lexical fact beside its existing syntax
validators, without adding an evaluator responsibility or a file. Retain that
unit for this bounded correction; defer broader XPath directory grouping to a
checkpointed parse/navigation responsibility campaign. Existing decomposition
debt is not erased by either retain disposition.

## Reproduction and limits

Run `scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/sort_sort36'` using the
unchanged OASIS CD04 archive, SHA256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
No fixture, expected output, comparator, exclusion, vendor revision, DTD
profile, acquisition authority, unsafe surface, or public contract changes.
`sort36` is not credited until its actual composed key executes and compares
exactly. A mixed-document union investigated during this tranche remains
unsupported under the deferred cross-document ordering boundary.

Final replay conserves 2,692 initialized / 478 initialization failures,
2,643 successful executions / 49 execution failures, 2,491 exact XML
comparisons, 14 mismatches, four comparator gaps, and 423 / 431 expected-error
credits. Direct-DTD attribution remains 134. `sort36` reports located
`unsupported/FXXP1001` instead of false `invalid/XPST0003`; the dormant
`ForwardComp__91853` case again compares exactly after proper arity deferral.
The instantiated `ForwardComp__91854` case reports `invalid/XPST0017` during
execution, not initialization. No new pass is claimed.

Both new focused compiler controls pass. Final workspace formatting, strict
Clippy, tests, documentation, unsafe-surface, conformance-source, and local
Markdown-link gates pass. Core tests report 1,583 passed and 57 manual probes
ignored; native, WASM, and worker suites report 22, four, and eighteen passed
respectively. Link verification checks 1,977 Markdown files. The official
`wasm32-unknown-unknown` all-features check passes. No runtime performance
study or general composed-expression support claim follows.
