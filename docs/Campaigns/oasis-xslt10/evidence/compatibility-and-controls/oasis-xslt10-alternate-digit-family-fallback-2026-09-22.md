# OASIS XSLT 1.0 Alternate Digit-Family Fallback

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Can the XSLT 1.0 compatibility path honor a declared alternate `zero-digit`
for a picture written only with ASCII zero placeholders without changing the
modern formatter or regressing pictures that already use the declared digit
family?

## Method

- Inventory all 16 cases at the runtime `FXRT1007` formatting frontier.
- Separate 12 deliberate execution-error cases from four success cases.
- Keep the rule behind the compiled XSLT 1.0 compatibility flag.
- Use ASCII `0` as a compatibility fallback only when the picture contains no
  active placeholder from the declared `digit`/`zero-digit` family.
- Preserve quoted characters and pictures that already contain declared
  placeholders.
- Differentially exercise the OASIS gray-area Lotus picture that mixes the
  declared family with a trailing ASCII zero.
- Rerun the unchanged 3,173-case catalog.

## Result

The three unchanged Microsoft `FormatNumber_DecimalFormatZeroDigit*` cases
become exact, producing `aaaa`, `abcd`, and `abcdefghij`. The already-passing
Lotus `numberformat_numberformat34` retains its declared-family placeholders
and literal trailing `0`; the initial unconditional prototype was rejected
because it regressed this conserved case.

The conserved totals are 2,140 initialized cases, 995 initialization failures,
2,064 successful executions, 76 execution failures, 1,921 exact XML-semantic
matches, and 56 mismatches. The strict compatibility lower bound is
`1,921 / 3,173 = 60.54%`.

## Boundaries

- This is a compile-selected XSLT 1.0 compatibility fallback, not a change to
  the modern formatting path.
- A picture using the declared digit family remains authoritative; ASCII zero
  is not globally reclassified.
- The remaining runtime formatting frontier is dominated by deliberate invalid
  pictures and invalid dynamic format names. Those require typed error
  classification, not permissive output recovery.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_ascii_zero_picture_selects_the_declared_digit_family
./scripts/measure-oasis-xslt10.ps1 -TraceCase FormatNumber_DecimalFormatZeroDigit
./scripts/measure-oasis-xslt10.ps1 -TraceCase numberformat_numberformat34
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier unsupported/FXRT1007/FXRT1007
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
