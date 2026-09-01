# XSLT30 shallow-skip mode policies — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` cases `mode-1417`, `mode-1419`, and
`mode-1421` exercise `on-no-match="shallow-skip"` over the suite's complete
external `mode-14.xml` source.

- `mode-1417` combines named shallow-skip descent with one explicit
  `book|bktlong|bktshort` copy rule.
- `mode-1419` proves `xsl:next-match` falls through to the active mode's
  shallow-skip built-in before a descendant text rule contributes `Genesis`.
- `mode-1421` composes shallow-copy mode `s` with shallow-skip mode `t`,
  preserving the source structure while producing empty copied `v` elements.

## Implemented semantics

- The mode compiler retains `shallow-skip` as an explicit named or unnamed
  policy.
- Unmatched document and element nodes apply templates to their children in
  the same active mode with the normal focus, parameter, budget, and
  cancellation path.
- Unmatched text, attribute, comment, and processing-instruction nodes produce
  an empty sequence.
- `xsl:next-match` reaches that same built-in policy when no lower-ranked
  explicit rule remains.
- Temporary-tree traversal applies the same skip behavior instead of copying
  unmatched temporary text. A focused control traverses unmatched temporary
  elements, reaches one explicit descendant rule, and drops surrounding text.

The one-step union alternatives in `mode-1417` reuse the existing expanded-name
alternative representation but retain exact-name default priority. Multi-step
alternatives retain path default priority; this tranche does not collapse those
distinct standard ranks.

## Result

All three unchanged native XML assertions pass. The conserved 169-case mode
denominator advances from 70 to 73 passes, retains 45 profile exclusions, and
reduces visible default not-run cases from 54 to 51. Across the 11 conserved
XSLT30 denominators, the totals become 267 passes, 3 engine-unsupported cases,
50 profile exclusions, and 211 visible default not-run cases.

## Boundary retained

`mode-1413` remains visible. Its matched attribute template must contribute an
attribute result into shallow-copy's element construction. FastXSLT's current
private result sequence represents elements, text, and processing instructions
while element attributes remain a separate owned collection. This tranche does
not introduce a special arithmetic-only attribute path or let a private result
type silently settle the broader result-sequence boundary.
