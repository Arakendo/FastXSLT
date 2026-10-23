# XSLT30 `template-004` Attribute Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `template`, case `template-004` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The unmodified case selects `@attribute1` from the matched `doc` element,
applies templates in `mode1`, matches the same unprefixed attribute pattern in
that mode, and emits its string value. A deliberately failing default-mode
attribute rule remains unselected.

Selection uses the XDM owner's attribute collection. It does not add attributes
to `Document::children` or reinterpret attribute document order as child order.
Each visited attribute is charged to the XPath node-visit work domain before
name comparison.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 5 |
| Engine unsupported and not run | 1 |
| Total | 6 |

Only `template-005`, which requires named templates, parameters, conditional
expressions, calls, and recursion, remains unsupported in this test set.

## Claim boundary

This case proves one unprefixed abbreviated attribute-axis selection and exact
attribute pattern. It does not establish wildcard, namespace-qualified,
predicate, general axis, pattern-priority, or complete mode conformance.
