# OASIS XSLT 1.0 Sort Data-Type Invalid Effective Values

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can FastXSLT distinguish invalid unqualified `xsl:sort/@data-type` effective
values from syntactically valid prefixed extension data-type names?

## Method

- Retain `text` and `number` as the admitted built-in values.
- Classify empty, unqualified unknown, and malformed static values as invalid
  `XTDE0030`.
- Preserve a syntactically valid prefixed QName as unsupported `FXST1044`; the
  XSLT 1.0 specification deliberately leaves extension data-type behavior
  implementation-defined.
- Trace the complete prior OASIS `FXST1044` frontier through the conserved
  3,173-case runner.

## Result

All four unchanged OASIS cases in the prior frontier now report invalid
`XTDE0030`:

- `Microsoft/Errors_err091#1` (`unknown`);
- `Microsoft/Sorting__77550#1` (empty);
- `Microsoft/Sorting__77552#1` (`foo`); and
- `Microsoft/Sorting__77556#1` (`number;text`).

All four were already credited expected-error cases, so the conserved totals
remain 2,210 initialized cases, 2,157 successful executions, and 2,009 / 3,173
(63.32%) exact expected-result matches. The corpus-visible `FXST1044` frontier
is eliminated without claiming extension sort support.

## Boundaries

- Prefixed extension data-type QNames remain explicitly unsupported.
- Dynamic variable-only built-in selection is unchanged; wider dynamic AVTs
  remain unsupported.
- No collation, language-sensitive, or extension comparison behavior is added.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt sort_controls_fold_exact_literal_avts
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'invalid/XTDE0030/XTDE0030'
./scripts/verify.ps1
```
