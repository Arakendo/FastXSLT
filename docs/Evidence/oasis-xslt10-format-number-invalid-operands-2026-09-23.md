# OASIS XSLT 1.0 `format-number()` Invalid Operands

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can the two remaining runtime `FXRT1007` cases be classified as invalid
expressions or names without pretending that FastXSLT implemented new
formatting behavior?

## Method

- Reject a missing first `format-number()` argument during compilation as
  invalid XPath syntax `XPST0003`.
- Resolve a variable-supplied third argument as a QName at runtime and report
  invalid or unavailable names as `XTDE1280`.
- Retain valid dynamic decimal-format selection, formatter behavior, and the
  existing unsupported boundary for genuinely unimplemented operand shapes.
- Trace both unchanged OASIS expected-error cases through the complete
  3,173-case measurement.

## Result

`Microsoft/XSLTFunctions__without1stParameter#1` moves from an unsupported
execution to invalid initialization `XPST0003`.
`Microsoft/XSLTFunctions__crahedOnEmptyVariable#1` reports invalid dynamic
`XTDE1280` when its empty variable cannot name a decimal format. No
`FXRT1007` cases remain in the current complete-catalog execution frontier.

Because one expected error now occurs during initialization rather than
execution, initialized cases move from 2,210 to 2,209, successful executions
remain 2,156, execution failures move from 54 to 53, initialization failures
move from 925 to 926, and observed expected errors move from 29 execution / 393
initialization to 28 execution / 394 initialization. The strict exact-result
lower bound remains 2,008 / 3,173 (63.32%).

## Boundaries

- No new number conversion, picture, or decimal-format feature is admitted.
- A valid QName naming an unavailable decimal format also remains an invalid
  runtime request; it is not silently redirected to the default format.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features format_number
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/XSLTFunctions__crahedOnEmptyVariable#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/XSLTFunctions__without1stParameter#1'
./scripts/verify.ps1
```
