# OASIS qualified variable-path evidence

Date: 2026-10-01

## Result

The unchanged OASIS CD04 `Microsoft/BVTs_bvt073#1` now initializes, executes,
and compares exactly. The case combines qualified global variables, qualified
template parameters, nested iteration, and recursive template application.
The strict lower bound rises from 2,469 to 2,470 exact comparisons.

The bounded XSLT 1.0 variable-path compiler previously recognized only an
unqualified NCName before `/`, although declarations and ordinary variable
references already normalized qualified names. It now accepts a lexical QName
and uses that existing static namespace normalizer before retaining the typed
relative path. Runtime binding lookup, lexical scope, source-node ownership,
work charging, and cancellation use their unchanged shared implementations.

Prefix aliases resolve to the same expanded binding identity; different
namespace URIs do not collide merely because their local names match.
Unbound prefixes fail at compilation with existing `FXST0012 / invalid`
diagnostics and stylesheet provenance. This does not admit general variable
expressions, new path grammar, cross-document node unions, or a public binding
representation.

## Conserved measurement

The hash-verified complete local OASIS CD04 sweep reports:

- 3,173 catalog cases;
- 2,470 exact XML comparisons (77.84%);
- 2,667 initialized and 2,620 successfully executed;
- 503 initialization failures and 47 execution failures;
- 12 visible XML mismatches and four comparator gaps, unchanged;
- 423 / 431 expected-error credits, unchanged.

Direct DTD attribution remains 133 cases: 30 internal parses, 94 sealed-external
parses, and nine unsupported. Production DTD denial and corpus admission are
unchanged. No upstream fixture or expected result was modified.

## Verification and cohesion

Focused execution verifies `ABB`: namespace-alias access to a global node
binding, isolation from a different-namespace binding with the same local name,
and qualified parameter lookup inside nested iteration. A focused compiler
test checks unbound-prefix category, code, and source-resource provenance.

ADR-0004 review: the value-expression compiler is approximately 3,650 lines
with ten direct sibling files. Retain this seven-line behavioral addition in
the existing variable-path compiler: it composes an existing QName normalizer
and typed path, adds no state or new responsibility, and has no runtime dispatch
or ownership cost. Broader variable-expression grammar remains a named future
extraction opportunity; it is not introduced by this repair.

Reproduce with `scripts/measure-oasis-xslt10.ps1`, optionally
`-TraceCase 'Microsoft/BVTs_bvt073#1'`, and the focused test filter
`qualified_variable_path`. Run `scripts/verify.ps1` for all workspace gates.

All gates passed: formatting, strict Clippy, workspace tests (1,421 core
tests passed; 34 manual probes ignored), documentation, Markdown links,
unsafe-surface checks, and conformance source/inventory checks.
