# XSLT30 `conflict-resolution-0901` Parent/Child Patterns

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0901`
- Stylesheet: `conflict-resolution-0901.xsl`
- Environment: shared `conflict-resolution-09` principal source
- Dependency: `spec=XSLT10+`

## Method

The metadata-driven apply-templates helper resolves the shared source,
stylesheet, and asserted XML from the pinned suite. The admitted bytes execute
through a bounded sealed snapshot and identified batch of one without ambient
filesystem access after admission.

This is a conservation case over existing typed semantics rather than a new
syntax shortcut. The `//b` selection uses the leading-descendant origin already
shared with direct XPath evidence and returns all six `b` descendants in
document order. The compiled `doc/a/b` and `doc/z/b` patterns retain separate
three-step paths. For each candidate, selection walks to the required context
and evaluates the typed path; it does not inspect lexical pattern strings.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0901` | `111222` | equal | passed |

The first three descendants match `doc/a/b` and emit `1`; the final three match
`doc/z/b` and emit `2`, preserving source document order.

## Claim boundary

This evidence covers the exact unnamespaced child-step patterns and
leading-descendant selection used by the case. It does not admit general XSLT
pattern grammar, arbitrary descendant patterns, namespaces, predicates in the
match paths, union patterns, ambiguity behavior, or the complete 52-case
apply-templates denominator.

The adjacent `0501–0503` cases remain larger work because they require
`current()` or range-variable predicate semantics plus nested `xsl:copy` and
constructed attributes. The `0701–0802` cases require deliberate static
`xpath-default-namespace` and current-mode propagation rather than case-shaped
pattern shortcuts.
