# OASIS wildcard-subtraction evidence

Date: 2026-10-01

## Result

The unchanged OASIS CD04 `Lotus/select_select26#1` now initializes, executes,
and compares exactly. Its `@*-5` selection subtracts five from the numeric
value of the first attribute, yielding `<out>15</out>`.

The shared arithmetic splitter now distinguishes a wildcard step from a
multiplication operator after `@`, `/`, or `::`, and at the start of an operand.
A minus following such a wildcard is binary subtraction rather than a name
hyphen or unary sign. Ordinary multiplication by a negative operand, such as
`n*-5`, remains multiplication. Hyphenated names remain names.

This is a token-boundary repair, not a new numeric evaluator. Compiled
first-node versus zero-or-one conversion, exact arithmetic, work charging,
cancellation, source identity, and diagnostic behavior remain unchanged.

## Conserved measurement

The complete hash-verified local OASIS CD04 sweep reports:

- 3,173 catalog cases;
- 2,471 exact XML comparisons (77.88%), up one;
- 2,668 initialized and 2,621 successfully executed, each up one;
- 502 initialization failures and 47 execution failures;
- 12 visible XML mismatches and four comparator gaps, unchanged;
- 423 / 431 expected-error credits, unchanged.

Direct DTD attribution remains 133 cases: 30 internal parses, 94 sealed-external
parses, and nine unsupported. Production DTD denial, acquisition authority,
fixture bytes, expected results, and comparator policy are unchanged.

## Verification and cohesion

Lexical controls cover `@*-5`, spaced subtraction, `attribute::*-5`, `child/*-5`,
`*-5`, ordinary negative multiplication, standalone wildcard paths, and
hyphenated names. A compiled transform runs under both XSLT 1.0 and 3.0 and
verifies `15|15|4|-45|-100`. Existing cardinality regressions remain intact.

The approximately 930-line numeric owner retains its existing responsibility:
typed arithmetic parsing and evaluation. The small lexical helper introduces
no public representation, new module, ownership boundary, runtime state, or
unsafe surface. The dense XPath directory retains its existing named numeric
owner; this repair does not justify a structural move during semantic work.

Reproduce with `scripts/measure-oasis-xslt10.ps1`, optionally
`-TraceCase 'Lotus/select_select26#1'`, and `scripts/verify.ps1`.

All verification gates passed: formatting, strict Clippy, workspace tests
(1,423 core tests passed; 34 manual probes ignored), documentation,
unsafe-surface and conformance checks. Markdown links were rechecked after
the evidence and roadmap updates.
