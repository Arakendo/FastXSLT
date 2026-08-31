# XSLT30 mode-1405 text-only-copy built-in rules — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` case `mode-1405` exercises a named initial
mode whose `on-no-match` policy is `text-only-copy` against the native
`mode-14.xml` source.

## Implemented semantics

- The mode declaration compiler retains `text-only-copy` as an executable
  mode-local policy for named and unnamed modes.
- Unmatched document and element nodes recursively apply templates to their
  children in the same active mode.
- Unmatched text and attribute nodes contribute their string value.
- Unmatched comment and processing-instruction nodes contribute nothing.
- Child dispatch preserves focus position and size, parameters, cancellation,
  work accounting, and the active mode through the existing reference path.

The ordinary built-in template behavior now calls the same complete safe
implementation, avoiding a second semantic path for the equivalent untyped
tree behavior.

## Result

The unchanged native case satisfies its `starts-with(normalize-space(.), ...)`
assertion over the complete 9 KB source. The test harness evaluates that exact
assertion shape without altering the stylesheet, source, or expected value.

The conserved 169-case mode denominator advances from 65 to 66 passes, retains
45 profile exclusions, and reduces visible default not-run cases from 59 to
58. Across the 11 conserved XSLT30 denominators, the visible totals become 260
passes, 3 engine-unsupported cases, 50 profile exclusions, and 218 default
not-run cases.

## Boundary exposed

Adjacent `mode-1407` and `mode-1409` also depend on `text-only-copy`, but their
explicit templates require the currently unsupported XPath expression
`upper-case(.)`. They remain visible default not-run cases until that separate
function-expression slice is admitted.
