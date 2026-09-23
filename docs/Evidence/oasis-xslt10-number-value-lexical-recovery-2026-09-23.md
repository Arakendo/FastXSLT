# OASIS XSLT 1.0 Number-Value Lexical Recovery

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can `xsl:number value` preserve the XSLT 1.0 lexical result for a value that
converts to `NaN`, without changing the modern numbering path or the formatting
of finite values?

## Method

- Record XSLT 1.0 compatibility on the private compiled number instruction.
- Retain both the canonical numeric lexical and, for an invalid literal or
  context item, the original string value.
- In XSLT 1.0 mode only, emit that original string when numeric conversion
  produces `NaN` and do not apply the number-format token or punctuation.
- Keep finite values, including zero and values below `0.5`, on the existing
  number-format path.
- Keep the modern path on its prior canonical `NaN` plus format behavior.
- Test the cross-version distinction and rerun the unchanged 3,173-case OASIS
  catalog.

## Result

The unchanged cases `Microsoft/Number_NaNOrInvalidValue#1`,
`Microsoft/Number_ValueAsNodesetTest1#1`, and
`Microsoft/Number_ValueAsEmptyNodeset#1` become exact expected-result matches.

The complete sweep still initializes 2,195 cases and executes 2,142
successfully. Exact XML-semantic matches rise from 1,993 to 1,996, mismatches
fall from 65 to 62, execution failures remain 53, and comparator-unsupported
results remain 75. The strict complete-catalog lower bound is
`1,996 / 3,173 = 62.91%`.

An attempted adjacent change that emitted number-format punctuation for an
empty patterned single-number list was rejected: it fixed one Microsoft case
but regressed two unchanged Lotus cases. The retained implementation therefore
changes only the consistently evidenced `NaN` lexical rule.

## Boundaries

- Compatibility is selected at compilation; execution does not inspect the
  stylesheet version.
- The modern path remains a differential oracle and still formats canonical
  `NaN` as before.
- Finite rounding and decimal/alphabetic/Roman formatting are unchanged.
- The work charge, cancellation point, result construction, source identity,
  and resource authority are unchanged.
- This does not admit a general XPath expression evaluator for `xsl:number`
  values.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_number_applies_bounded_decimal_format_tokens
cargo test -p fastxslt --all-features modern_number_value_retains_formatted_non_number_oracle
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Number_NaNOrInvalidValue#1'
./scripts/verify.ps1
```
