# OASIS XSLT 1.0 Default Decimal `format-number()` Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Corpus | OASIS XSLT/XPath 1.0 CD04, locally acquired and hash-verified |
| Scope | Source-free numeric operands and static pictures using the default decimal format |
| Disposition | Twenty-one execution failures become exact expected-result matches |

## Change

The existing typed `FormatNumberExpression` now evaluates bounded source-free
numeric operands, including decimal literals, multiplication, division,
`round()`, non-finite values, and numeric string literals. Static default-format
pictures support required and optional digits, repeated grouping separators,
decimal
rounding, positive/negative subpictures, literal affixes, percent, and per-mille.

The implementation validates the ordering of required and optional digit
placeholders. The two corpus error cases `#.#0` and `0#.#` continue to produce
execution errors, so broader formatting did not turn invalid pictures into
successful results. Named `xsl:decimal-format`, dynamic path arithmetic,
exponent notation and broader compatibility
rules remain explicit boundaries.

## Measurement

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,269 | 1,269 | 0 |
| Executed successfully | 1,075 | 1,096 | +21 |
| Expected-result XML matches | 962 | 983 | +21 |
| XML comparison mismatches | 78 | 78 | 0 |
| Execution failures | 194 | 173 | -21 |
| Expected execution errors observed | 15 | 15 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |

The strict standard-operation lower bound is now `983 / 2,742 = 35.85%`;
the deliberately conservative all-catalog ratio is `983 / 3,173 = 30.98%`.
This is compatibility evidence, not an XSLT 1.0 conformance claim.
