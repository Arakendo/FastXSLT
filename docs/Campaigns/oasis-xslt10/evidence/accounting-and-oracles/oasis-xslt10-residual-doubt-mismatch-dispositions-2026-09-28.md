# OASIS XSLT 1.0 Residual Doubt-Mismatch Dispositions

Date: 2026-09-28  
Status: Verified corpus dispositions  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

After the locale-invariant numbering tranche, only five successfully executed
XML comparisons remained mismatches. Every one carries substantive archive
doubt or gray-area metadata. They were reviewed individually rather than left
as an unexplained red remainder or converted into pass credit.

## Dispositions

### Unusable archival references

- `Microsoft/Number__84692#1` produces the complete table constructed by the
  stylesheet. The archival reference contains only an empty `TABLE`, and its
  doubt entry explicitly states that sub-elements are missing from `REF_OUT`.
- `Microsoft/XSLTFunctions__minimalValue#1` and
  `Microsoft/XSLTFunctions__minimumValue#1` expect scientific notation for an
  extremely small finite XPath number. Their doubt entries call out that exact
  reference behavior. XPath 1.0 `string(number)` uses a decimal representation
  and explicitly does not use exponential notation; FastXSLT retains that
  conversion.

These three cases receive exact `unusable-reference-result` dispositions and
no pass credit.

### XSLT 1.0 discretionary recovery

`Microsoft/Number__91028#1` and `Microsoft/Number__91029#1` exercise the
historically disputed interaction of `level="any"` or `level="multiple"` with
`from`. Their gray-area metadata selects blank output for named
`special-from-case-any` and `special-from-case-multiple` choices and records
that the matter was referred for an erratum. FastXSLT retains its existing,
internally consistent recovery and gives both cases exact
`xslt10-discretionary-policy-excluded` dispositions. Neither receives pass
credit.

## Measurement effect

The strict exact-match lower bound remains `2,224 / 3,173 = 70.09%`.
Initialized and successfully executed totals remain 2,435 and 2,387. Visible
XML mismatches fall from five to zero; unusable archival reference results rise
from 57 to 60; and XSLT 1.0 discretionary-policy exclusions rise from three to
five.

The result is a conserved catalog with no unexplained successfully executed XML
mismatch. It is not a claim that excluded cases pass and does not change the
3,173-case denominator.

## Boundaries

- No engine behavior, expected bytes, or upstream metadata were changed.
- A doubt entry alone does not justify exclusion; each disposition names the
  concrete reference defect or discretionary standard behavior.
- Scientific notation remains available only where a modern selected semantic
  path requires it; this record preserves the XPath 1.0 conversion rule.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
cargo test -p fastxslt --all-features oasis_xslt10_discretionary_policy_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```

## Standards source

- [W3C XPath 1.0 section 4.2](https://www.w3.org/TR/1999/REC-xpath-19991116#string-functions)
- [W3C XSLT 1.0 section 7.7](https://www.w3.org/TR/1999/REC-xslt-19991116#number)
