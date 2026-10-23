# OASIS XSLT 1.0 `format-number()` Invalid-Picture Classification

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

When the admitted formatter proves that a resolved `format-number()` picture
violates picture grammar, does FastXSLT report invalid input rather than an
unsupported engine capability?

## Method

- Separate invalid resolved-picture grammar from still-unsupported numeric
  conversion and decimal-format name resolution.
- Map invalid pictures to structured dynamic diagnostic `XTDE1310`.
- Retain the existing parser, decimal-format rules, exact formatter, limits,
  and runtime evaluation path.
- Exercise focused invalid pictures and trace the complete 3,173-case OASIS
  measurement.

## Result

Eleven execution-frontier cases move from unsupported `FXRT1007` to invalid
`XTDE1310`; the `FXRT1007` frontier falls from 13 cases to 2. Ten of the moved
cases are catalog expected-error scenarios. The eleventh,
`Microsoft/XSLTFunctions__specialCharInPattern#1`, is cataloged as a normal
scenario even though its purpose and source fixture describe `;#` as an
invalid picture that should raise an error. It therefore remains visibly
uncredited rather than being forced into a pass.

The harness already counted the expected-error executions as observed errors,
so aggregate totals do not move: 2,210 cases initialize, 2,156 execute
successfully, and 2,008 / 3,173 (63.32%) match an admitted expected result
exactly. This tranche improves diagnostic and frontier truth, not breadth.

## Boundaries

- No new picture form or formatter algorithm is admitted.
- Unsupported numeric conversion and dynamic decimal-format resolution remain
  explicit `FXRT1007` boundaries.
- The contradictory normal-scenario catalog entry receives no pass credit.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features invalid_picture
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Errors_err036#1'
./scripts/verify.ps1
```
