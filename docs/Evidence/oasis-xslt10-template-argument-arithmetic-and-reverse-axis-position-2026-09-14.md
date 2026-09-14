# OASIS XSLT 1.0 template-argument arithmetic and reverse-axis position -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an `xsl:with-param` reuse the existing XSLT 1.0 binary numeric evaluator,
and can its result select the correct proximity position on a reverse axis?

## Implemented slice

Template arguments now retain the existing typed binary-numeric expression
when that bounded compiler admits the `select` expression. Runtime evaluation
uses the same controlled arithmetic path as other XSLT 1.0 numeric consumers
and passes its result through the existing atomic parameter frame. No second
arithmetic evaluator or template-specific conversion rule was introduced.

That change moved two recursive OASIS cases through `$this + 1` and exposed a
separate semantic defect in `preceding::text()[$this]`. The general location
path evaluator correctly normalizes its final node sequence into document
order, but the numeric predicate had then selected from that normalized order.
XPath 1.0 predicate proximity positions instead use the axis direction:
`preceding`, `preceding-sibling`, `ancestor`, and `ancestor-or-self` are reverse
axes.

The repair is deliberately narrow. The existing variable-position compiler
admits only a single path step. That retained path can now report whether its
sole axis is reverse, and the compatibility evaluator selects from the reverse
of the document-ordered result in that case. The reusable path result remains
document ordered, and general multi-step predicate semantics are not inferred.

## Corpus result

Both cases that had reached `$this + 1` now execute and match their unchanged
expected results:

| Case | New disposition |
| --- | --- |
| `Lotus/position_position78` | Exact expected-result match |
| `Microsoft/Miscellaneous_TestOfPrecedingAxis` | Exact expected-result match |

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,698 | 1,700 | +2 |
| Initialization failures | 1,437 | 1,435 | -2 |
| Executed successfully | 1,515 | 1,517 | +2 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,382 | 1,384 | +2 |
| XML comparison mismatches | 106 | 106 | 0 |

The exact compatibility lower bound is now
`1,384 / 2,742 = 50.47%` for standard-operation cases and
`1,384 / 3,173 = 43.62%` for the complete catalog.

## Boundaries

This tranche does not admit arbitrary template-argument expressions, a general
predicate AST, variable-rooted paths, or multi-step variable-position paths.
It does not change final XPath node-sequence ordering. Reverse-axis proximity
is applied only while choosing the one node selected by the already-bounded
single-step compatibility expression.

The numeric argument remains a private `xs:double` value, consistent with the
XPath 1.0 number model. No public expression, plan, or parameter representation
is selected.

## Verification

- A focused runtime test proves `$one + 1` crosses a named-template parameter
  boundary and serializes as `2`.
- A focused runtime test proves both `$p` and `position() = $p` select the
  nearest `preceding::text()` node rather than the first node in document order.
- The complete 3,173-case measurement moves exactly the two affected cases
  from initialization failure to exact result without a new mismatch or
  execution failure.
- Formatting, strict Clippy, the complete workspace suite, documentation, link,
  unsafe-surface, and corpus-inventory checks pass through `scripts/verify.ps1`.
