# OASIS XSLT 1.0 Positional Namespace Identity

The 2026-10-07 continuation gains unchanged `Lotus/idkey_idkey31#1`.
OASIS CD04 exact XML comparisons advance from 2,499 to 2,500 / 3,173
(78.79%). This is corpus compatibility evidence, not general conformance.

## Shared semantics and ownership

The existing private legacy positional identity plan now distinguishes ordinary
path unions from a terminal namespace selection. For
`generate-id((descendant-or-self::*/namespace::*)[position() = $variable])`,
the existing qualified-node evaluator supplies normalized selection order
before the whole-set position predicate. The existing atomic position conversion
and principal-origin identity guard remain shared with ordinary union identity.
The namespace string encoder is reused, not duplicated. Source origin, owning
element and prefix remain namespace identity; equal binding URIs do not merge
different elements' occurrences.

Scalar output and local atomic-string bindings share this consumer. No namespace
value escapes into runtime frames or prepared/compiled data. Effective whitespace
views retain their existing navigation and identity behavior. Qualified traversal
and namespace encoding retain their existing charges before retained allocation.
Compiled retention includes the typed selection, boxed qualified path, owned
path payload, variable name and provenance. Ordinary union plans keep their
existing selection buffer and normalization path.

Modern expression composition, mixed positional namespace unions, namespace-valued
variables, temporary/secondary identity strings, cross-document ranking, global
or key-use eligibility, and public identity spelling are not admitted. AR-0029's
preprocessing-edition decision remains deferred; no whitespace behavior changes.

## Controls

Focused compiled controls compare each normalized position against direct
namespace identity, distinguish inherited occurrences on different elements,
compare effective views with the complete derivation, and run four concurrent
invocations against one prepared source. Numeric/string positions, empty and
out-of-range selection, exact/one-less operation budgets, charged cancellation
and subsequent reuse are covered. Unbound and non-atomic positions retain
located invalid/unsupported diagnostics. Modern and mixed-union expressions
remain explicitly unsupported. The unchanged three ordinary positional-union
tests continue to cover boolean/host-parameter conversion and frame isolation.
Hand-authored controls earn no corpus credit.

The compiler change stays in its private grouped-selection owner; runtime
position conversion and namespace encoding retain their existing private owners.
Composition roots only route the typed selection. Existing source-unit pressure
is not discharged by this narrow change. No new crate, backend, public type,
unsafe code, acquisition authority or cross-invocation cache is introduced.

## Reproduction and conserved accounting

Run `scripts/measure-oasis-xslt10.ps1 -TraceCase idkey31` against unchanged
OASIS CD04 archive SHA256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
The result `<out>Number of IDs accumulated: 10</out>` compares exactly through
the established XML comparator. No fixture, reference, comparator, exclusion,
vendor revision or DTD policy changes.

The complete replay reports 2,700 initialized, 470 initialization failures,
2,652 successful executions, 48 execution failures, 2,500 exact comparisons,
14 mismatches, four comparator gaps and 423 / 431 expected-error credits.
The direct-DTD attribution total remains 134. Another 39 exact
cases are needed to reach at least 80% (2,539 / 3,173).

All three new positional-namespace controls and the three unchanged ordinary
positional-union controls pass. The complete workspace verification passes
formatting, strict Clippy, tests, documentation, unsafe-surface, local links
and conformance inventories: 1,660 core tests passed with 60 manual probes
ignored; native, WASM and worker suites pass 22, four and eighteen tests.
Local links are checked in 1,984 Markdown files. The official all-features
`wasm32-unknown-unknown` adapter check and `git diff --check` also pass.
No performance claim follows from this tranche.
