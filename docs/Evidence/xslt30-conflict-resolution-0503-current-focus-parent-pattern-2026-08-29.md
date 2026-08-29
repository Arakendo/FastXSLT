# XSLT30 `conflict-resolution-0503` Current-Focus Parent Pattern

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0503`
- Stylesheet: `conflict-resolution-0503.xsl`
- Source: inline `conflict-resolution-05` environment

## Representation and execution

The pattern `*[name()=name(current())]/*` retains two distinct focus roles.
The final `*` supplies the candidate node being matched and therefore the value
of `current()`. The preceding step and its predicate are evaluated against the
candidate's parent. The predicate succeeds only when the parent and final
candidate have the same lexical element name.

Compilation lowers that exact form to a typed
`ElementWithSameNamedParent` relation rather than treating it as the
same-named-child scan admitted by `0501` and `0502`. Runtime evaluation charges
the parent visit and explicitly rejects namespaced candidates until lexical
QName prefix identity can be represented faithfully.

The nested `a` has an `a` parent, selects the predicate template, and is copied
with `parent-recursive="yes"`. The sibling `b` has an `a` parent and selects the
ordinary wildcard template.

## Result

| Case | Expected distinguishing result | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0503` | nested `a` has `parent-recursive="yes"` | semantically equal | passed |

## Claim boundary

This evidence admits only the exact unnamespaced parent/current pattern above.
It does not admit general multi-step pattern predicates, arbitrary `current()`
expressions, namespace-sensitive `name()` comparison, positional predicates,
or the more complex `1501` pattern.
