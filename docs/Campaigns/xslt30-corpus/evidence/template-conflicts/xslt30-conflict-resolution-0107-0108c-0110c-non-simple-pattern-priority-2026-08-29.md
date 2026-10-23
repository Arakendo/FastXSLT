# XSLT30 `0107/0108c/0110c` Non-Simple Pattern Priority

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Cases and stylesheets:
  - `conflict-resolution-0107` / `conflict-resolution-0107.xsl`
  - `conflict-resolution-0108c` / `conflict-resolution-0108.xsl`
  - `conflict-resolution-0110c` / `conflict-resolution-0110.xsl`
- Environment: embedded `conflict-resolution-01` principal source

## Method

The metadata-driven apply-templates helper resolves each selected case, shared
embedded source, stylesheet, and expected XML from the pinned test set. Every
source/stylesheet pair executes through its own bounded sealed snapshot and an
identified batch of one without ambient filesystem access after admission.

`0107` is a conservation case for compile-time priority. Its `doc/foo` path
pattern retains the non-simple default priority above exact `foo`, element
wildcard, and any-node fallbacks. Execution compares retained priorities rather
than reconstructing pattern categories in the dispatch loop.

The paired XSLT 3.0 cases add the exact attribute-presence pattern
`foo[@test]`. It matches only an unnamespaced `foo` element with an unnamespaced
`test` attribute and charges each inspected attribute as an XPath node visit.
The pattern shares the admitted non-simple priority with `doc/foo`. `0108c`
declares the path later, while `0110c` declares the predicate later, proving
source-order last-match selection in both directions.

A focused compiler control preserves the typed element/attribute names and
equal compiled priority, while an attribute value comparison such as
`foo[@test='true']` remains explicitly unsupported.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0107` | `Match of non-simple '/'` | equal | passed |
| `conflict-resolution-0108c` | `Match-of-non-simple '/'` | equal | passed |
| `conflict-resolution-0110c` | `Match-of-non-simple '[...]'` | equal | passed |

## Claim boundary

This evidence admits only the existing relative path-pattern form and the exact
ASCII, unnamespaced `element[@attribute]` presence form used here. It does not
admit general pattern predicates, attribute value comparisons, boolean
expressions, namespaces, wildcard predicates, union patterns, ambiguity
warnings, XSLT 1.0/2.0 recovery policy, or the complete denominator. Bounded
fractional priority is evidenced separately by `conflict-resolution-1701`.
