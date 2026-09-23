# OASIS XSLT 1.0 Number-Level Static Classification

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Does an unknown lexical value for `xsl:number/@level` remain visibly invalid
rather than being reported as an unsupported FastXSLT capability?

## Method

- Compile a focused XSLT 1.0 stylesheet containing
  `<xsl:number level="unknown"/>`.
- Require the compiler to classify the declaration as invalid with structured
  diagnostic `XTSE0020`.
- Trace the unchanged OASIS `Microsoft/Errors_err070#1` expected-error case
  through the complete 3,173-case measurement harness.

## Result

The focused stylesheet and unchanged OASIS case now report invalid
`XTSE0020`. The case was already counted as an initialization-time expected
error, so the conserved totals and strict exact-result lower bound do not
change: 2,210 cases initialize, 2,156 execute successfully, and 2,008 / 3,173
(63.32%) match an admitted expected result exactly.

The useful change is frontier honesty: an unrecognized enumeration value is
invalid input, not evidence of an unimplemented `xsl:number` level.

## Boundaries

- The accepted `single`, `multiple`, and `any` levels are unchanged.
- No new numbering semantics or error-recovery behavior is introduced.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features rejects_an_unknown_xsl_number_level_as_invalid
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Errors_err070#1'
./scripts/verify.ps1
```
