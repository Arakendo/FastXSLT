# OASIS XSLT 1.0 Number Letter-Value Static Classification

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Does an unknown lexical value for `xsl:number/@letter-value` remain visibly
invalid without misclassifying valid but unimplemented numbering behavior?

## Method

- Compile focused XSLT 1.0 stylesheets covering accepted `alphabetic` and
  `traditional` values, a valid but unsupported alphabetic reinterpretation of
  a Roman token, and the invalid value `unknown`.
- Require `unknown` to report invalid `XTSE0020` while the valid but
  unsupported reinterpretation remains `FXST1049`.
- Trace unchanged OASIS case `Microsoft/Errors_err069#1` and inventory the
  remaining `FXST1049` frontier through the complete 3,173-case runner.

## Result

The focused invalid stylesheet and unchanged OASIS case now report invalid
`XTSE0020`. The OASIS case already received initialization-time expected-error
credit, so the conserved totals do not move: 2,209 cases initialize, 2,156
execute successfully, and 2,008 / 3,173 (63.32%) match an admitted expected
result exactly.

The unsupported `FXST1049` frontier falls from five cases to four. Those four
remain materially different work: dynamic number-format evaluation, a Greek
alphabetic token, and two compound-token formats. The correction therefore
improves diagnostic honesty without claiming those semantics.

## Boundaries

- The accepted lexical values remain exactly `alphabetic` and `traditional`.
- Valid-but-unimplemented token reinterpretation remains unsupported rather
  than being mislabeled invalid.
- No dynamic format, non-Latin numbering, or broader compound-token behavior
  is admitted.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt number_admits_static_letter_values_only_when_existing_tokens_are_equivalent
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Errors_err069#1'
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'unsupported/FXST1049/FXST1049'
./scripts/verify.ps1
```
