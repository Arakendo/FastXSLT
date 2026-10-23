# OASIS XSLT 1.0 Filtered Union Identity

The 2026-10-04 continuation admits a narrow principal-source identity consumer
and gains the unchanged `Lotus/idkey_idkey30#1` case. The OASIS CD04 full replay
advances from 2,490 to 2,491 / 3,173 exact XML comparisons (78.51%). This is
corpus compatibility evidence, not a general XSLT 1.0 conformance claim.

## Semantics and ownership

The private typed plan recognizes `generate-id((path | path)[position() =
$variable])`, including reversed equality. Existing charged XPath union
evaluation supplies document order and duplicate removal before the whole-set
predicate. No per-branch positional filter or alternate path evaluator is used.

Scalar output and local variable bindings share one string evaluator. The local
binding is atomic, not a result-tree fragment: an empty identity therefore has
empty-string boolean behavior. Existing invocation-local safe COW binding owns
the value and preserves lexical isolation under ADR-0014 and ADR-0017.

Atomic string/untyped positions use XPath 1.0 numeric-string conversion;
numeric atomics retain numeric lexical conversion, including exponent notation.
Boolean equality converts every positive position to true, so true selects the
first normalized node and false selects none. Nonintegral, nonpositive,
nonfinite, out-of-range, and invalid numeric-string positions select none.
Non-atomic position values remain unsupported rather than silently applying
first-node conversion to an existential node-set comparison.

The common principal-identity guard is extracted from the namespace-string
consumer without changing its charges or diagnostics. Temporary and secondary
focus remain unsupported. Namespace-valued unions/variables, general modern
expressions, cross-document ranking, public identity spelling, global/key-use
eligibility, and resource authority are not expanded.

## Controls and accounting

Compilation retention counts the boxed typed plan, owned union paths, variable
name, and expression location; local bindings additionally count their name and
declaration provenance. Evaluation charges the function and conversion plus the
existing union traversal and normalization work. Result text and serialization
retain their existing separate charges.

Compiled controls cover overlapping/reversed branches, duplicate removal before
position, attribute/element order, reverse equality, scalar/local-binding parity,
empty-string truthiness, numeric/string/boolean host parameters, nested lexical
shadowing, visibility-view/complete-reference parity, exact and one-less work
limits, cancellation, located unsupported non-atomic and unbound positions,
complete atomic/sequence-frame clone parity, and four concurrent invocations.
Unsupported position values are checked by kind without copying their payload.
These first-party fixtures earn no corpus credit.

## Cohesion

The filtered-union compiler remains a private typed-family child. A private
runtime child owns the position conversion and identity consumer; parent units
only route and bind through existing contracts. The new test child owns this
consumer's invariants. Runtime value-evaluator and test directories have four
and five direct files respectively, so neither needs artificial grouping at
this checkpoint.
The already-pressured instruction/value compilers and runtime composition root
retain their existing ordered routing responsibilities; this change does not
erase their recorded decomposition debt. No new crate, public abstraction,
backend, unsafe code, or host policy is introduced.

## Reproduction

Run `scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/idkey_idkey30'` against
the unchanged CD04 archive with SHA256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
The observed output is `<out>Number of IDs accumulated: 18</out>`, matching the
unchanged reference through the established XML comparator.

The replay reports 2,692 initialized and 478 initialization failures, 2,643
successful executions and 49 execution failures, 2,491 exact XML comparisons,
14 mismatches, four comparator gaps, and 423 / 431 expected-error credits.
The direct-DTD attribution count remains 134. No fixture, expected output,
comparator, exclusion, vendor revision, or DTD/acquisition policy is changed.

The three focused compiled controls pass. Workspace formatting, strict Clippy,
tests, documentation, unsafe-surface, Markdown-link, and conformance-source
gates pass: 1,581 core tests passed with 57 manual probes ignored; native,
WASM, and worker suites report 22, four, and eighteen passed respectively.
Local link verification checks 1,976 Markdown files. The official
`wasm32-unknown-unknown` all-features check and final replay pass after the
last unsupported-value diagnostic change, conserving all counters above.
`git diff --check` also passes. No performance claim follows.
