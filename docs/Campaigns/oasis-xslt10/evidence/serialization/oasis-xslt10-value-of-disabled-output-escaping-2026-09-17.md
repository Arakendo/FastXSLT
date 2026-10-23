# OASIS XSLT 1.0 `xsl:value-of` Disabled Output Escaping -- 2026-09-17

Date: 2026-09-17  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The ordinary `xsl:value-of` compiler rejected `disable-output-escaping` before
distinguishing its values. The value `no` is semantically inert and requires no
escape-suppression representation, while `yes` conflicts with the current
semantic-result-tree boundary.

## Repair

`xsl:value-of` now accepts and validates the attribute. Missing or `no` uses the
ordinary value construction path, `yes` remains explicit unsupported
`FXST1060`, and any other lexical reports static `XTSE0020`. This does not add a
disable-output-escaping flag to result nodes or weaken serialization ownership.

## Corpus result

Unchanged Lotus `output_output07` becomes exact, raising the exact-result lower
bound from 1,517 to 1,518. Cases requesting `yes` remain at the named semantic
boundary.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,865 | 1,866 | +1 |
| Initialization failures | 1,270 | 1,269 | -1 |
| Executed successfully | 1,666 | 1,667 | +1 |
| Execution failures | 199 | 199 | 0 |
| Expected-result XML matches | 1,517 | 1,518 | +1 |
| XML comparison mismatches | 120 | 120 | 0 |
| XML comparator unsupported | 23 | 23 | 0 |
| Execution panics | 0 | 0 | 0 |

## Verification

Focused tests cover the inert `no` value, explicit unsupported `yes`, and an
invalid lexical. The complete sweep conserves all 3,173 identities and keeps
expected-error accounting unchanged at 403 initialization observations, 21
execution observations, and four doubt-annotated unexpected successes.
