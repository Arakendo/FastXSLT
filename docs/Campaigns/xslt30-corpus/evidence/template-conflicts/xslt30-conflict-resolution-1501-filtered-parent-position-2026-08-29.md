# XSLT30 `conflict-resolution-1501` Filtered Parent Position

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1501`
- Stylesheet: `conflict-resolution-1501.xsl`
- Source: inline principal source
- Assertion shape: `all-of` containing two XPath assertions

## Representation and execution

The pattern `*[name()=name(current())][2]/*` extends the parent/current
relation from `0503` with a filtered positional predicate. A final candidate
matches when:

1. its parent has the same lexical element name as the candidate; and
2. that parent is second among its element siblings whose lexical name also
   equals the final candidate's name.

The second condition is deliberately not approximated as “parent is the second
element child.” Compilation retains a typed
`ElementWithSameNamedParentAtPosition(2)` operation. Runtime evaluation charges
the parent visit and every inspected sibling and stays within the admitted
unnamespaced lexical-name domain.

The upstream result uses two XPath assertions rather than `assert-xml`. The
harness conserves both original expressions verbatim and applies a bounded
case-specific XML oracle equivalent to their conjunction. It does not claim a
general XPath assertion evaluator.

## Result

| Upstream assertion | Observed result | Disposition |
| --- | --- | --- |
| `/doc/a[2]/a[1][@parent-recursive="yes"]` | selected node carries the attribute | passed |
| `count(//@parent-recursive) = 1` | exactly one result attribute | passed |

## Claim boundary

This evidence admits only the exact positional current pattern and assertion
pair above. It does not admit general positional pattern predicates, arbitrary
`current()` expressions, namespace-sensitive `name()` comparison, numeric
position expressions other than this compiled constant, or general XPath
evaluation in the corpus assertion harness.
