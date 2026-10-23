# QT3 Axis Diagnostic Tail Production Path

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The remaining 42 selected cases from `prod/AxisStep.xml` after the 182-case
  successful count tranche.
- Two empty-origin paths, ten statically atomic path cases, 25 syntax failures,
  and five expressions invoked without a dynamic context item.

## Method and result

The unchanged expressions are wrapped in production XSLT. Empty-origin paths
compile into an owned location path, execute without visiting the supplied
document, and serialize `true`. Static path failures run through production
compilation and preserve the QT3-permitted `XPST0003`, `XPTY0019`, or
`XPTY0020` code.

The five context cases compile into a named template and execute through the
ordinary initial-template runtime without a principal source. Each reports
`XPDY0002`. The bounded context-required fallback remains explicitly
unsupported when a context exists and the general expression semantics are not
implemented.

All 42 comparisons pass. Together with the 182 successful axis-count cases,
the complete 224-case selected `AxisStep` denominator is production-backed.
Consequently all 1,170 current QT3 passes now execute or report their expected
diagnostic through production rather than relying solely on test-only semantic
helpers.

## Boundary

This does not admit arbitrary sequence construction or general evaluation of
the five context-dependent expressions. It admits their required missing-context
failure and keeps the positive-context behavior unsupported. Direct helpers
remain focused work-accounting and classification oracles.
