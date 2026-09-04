# QT3 `fn:exists` Initial Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Complete native test set `fn/exists.xml` with 58 cases.
- First-party overlay
  `corpus/overlays/qt3/exists-denominator-v0.toml`.

## Method

The same typed denominator and immutable upstream-expression path used for
`fn:empty` now conserves `fn/exists`. The shared safe evaluator computes
cardinality over the already validated bounded atomic-sequence grammar through
the private production expression,
supports prefixed and unprefixed calls plus `fn:not`, and charges every
executed XPath operation. `exists()` and `exists(1, 2)` remain native
`XPST0017` arity errors rather than collapsing into unsupported syntax.

The ninth direct function case exercises bounded literal `reverse()` before
the existence test. Invocation-clock expressions and the wider range/for/
predicate grammar remain unexecuted rather than receiving answers derived from
their expected assertions.

Every selected expression is compiled inside `xsl:value-of`, executed by the
ordinary runtime, serialized as text, and compared with its native assertion or
compile diagnostic. A workbench sentinel composes `exists` with `empty` through
the same host-facing engine path.

## Result

| Test set | Native cases | Selected and passed | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `fn/exists.xml` | 58 | 48 | 2 | 8 |

The 48 passes comprise 39 typed numeric singleton cases, two native arity
errors, six direct or negated literal-sequence cardinality cases, and one
reversed literal sequence. Two `XQ10+` cases are excluded from FastXSLT's
XPath-in-XSLT profile. Eight cases retain the visible default.

The audited QT3 subtotal is now 774 cases: 551 selected passes, 183 profile
exclusions, and 40 visible default not-run cases. The remaining 31,047 QT3
cases stay at catalog inventory only.

## Boundary

This evidence establishes the admitted source-free sequence-cardinality
operation. It does not claim arbitrary `fn:exists`, invocation-clock or
timezone functions, general `fn:remove`, range predicates, or XQuery `for` and
node constructors.
