# OASIS XSLT 1.0 `format-number()` Compact Additive Expression -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Microsoft `Number__10052` and `Number__10053` cases compile a
dynamic named decimal format and then evaluate `format-number(2-3.56, ...,
$format)`. FastXSLT already resolved the variable-valued format name and its
compiled decimal-format declaration, but its source-free numeric evaluator did
not recognize an additive operator without surrounding whitespace. Both cases
therefore stopped during execution at `FXRT1007`.

## Change

The private source-free numeric evaluator now recognizes top-level `+` and `-`
with normal lower precedence than multiplication and division. Its scanner:

- ignores operators inside parentheses and string literals;
- does not split a leading unary sign;
- does not mistake an exponent sign for an additive operator; and
- retains left associativity by selecting the final top-level additive
  operator before recursive evaluation.

This is shared expression behavior, not a corpus-case special form. Focused
tests cover the motivating XSLT 1.0 dynamic-format expression and modern
formatting with subtraction, multiplication precedence, and exponent signs.
Named decimal formats remain immutable compiled state, and runtime values still
come from the invocation-owned variable frame.

## Corpus result

Both unchanged Microsoft cases now execute and compare exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,417 | 2,417 | 0 |
| Executed successfully | 2,365 | 2,367 | +2 |
| Initialization failures | 753 | 753 | 0 |
| Execution failures | 52 | 50 | -2 |
| Exact expected-result matches | 2,205 | 2,207 | +2 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,207 / 3,173 = 69.56%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52. The standard-operation `FXRT1007` execution frontier
is empty again.

## Boundaries

- This tranche adds additive evaluation only to the already bounded,
  source-free numeric operand used by `format-number()`.
- It does not add source navigation, general XPath parsing, new decimal-format
  declarations, or new resource authority.
- Modern and XSLT 1.0 paths share the operator scanner but retain their existing
  numeric conversion and formatting profiles.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_dynamic_decimal_format_evaluates_compact_subtraction
cargo test -p fastxslt --all-features modern_format_number_retains_additive_precedence_and_exponent_signs
./scripts/measure-oasis-xslt10.ps1
```

## Normative references

- [XPath 1.0 section 3.5, Numbers](https://www.w3.org/TR/1999/REC-xpath-19991116/#numbers)
- [XSLT 1.0 section 12.3, Number Formatting](https://www.w3.org/TR/1999/REC-xslt-19991116/#format-number)
