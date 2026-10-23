# OASIS XSLT 1.0 Discretionary Numbering Policy

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Two successfully executed numbering cases select historical or disputed XSLT
1.0 behavior rather than exposing missing engine machinery:

- Lotus `numbering_numbering79` fixes one nonnumeric/NaN `xsl:number/@value`
  rendering as `(0)`. Its catalog and doubts metadata identify the behavior as
  discretionary or a gray area. FastXSLT retains its established lexical
  pass-through policy.
- Microsoft `Number__84687` expects an empty single-level number when the
  context node matches the effective count pattern but has no ancestor matching
  `from`. Its doubts metadata names this exact `special-from-case-single` gray
  area and records referral for an erratum. FastXSLT numbers the matching
  context node, the behavior already verified by unqualified Lotus
  `numbering_numbering20` and focused runtime coverage.

## Disposition

Both identities receive exact `xslt10-discretionary-policy-excluded`
dispositions. The selected compatibility behavior remains coherent, while the
archive's alternatives remain visible in the conserved denominator. Neither
case receives pass credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,126 exact XML comparisons out of 3,173 (67.00%). Visible XML
mismatches fall from 10 to 8, and the new policy bucket contains exactly two
cases.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_xslt10_discretionary_policy_exclusion_is_exact_and_bounded
cargo test -p fastxslt --all-features xslt10_single_number_honors_static_count_and_from_patterns
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
