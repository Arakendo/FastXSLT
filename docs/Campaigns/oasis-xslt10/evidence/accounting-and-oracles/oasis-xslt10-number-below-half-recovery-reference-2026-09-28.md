# OASIS XSLT 1.0 Number-Below-Half Recovery Reference

Date: 2026-09-28  
Status: Verified corpus disposition  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged `Microsoft/Number__91022#1` case executes successfully but the
archival expected result represents an extremely small positive
`xsl:number/@value` as `0`. FastXSLT instead emits the XPath string value of the
number.

This difference must not be resolved by changing engine behavior merely to
match the archive.

## Standards check

The substantive XSLT 1.0 erratum E24 states that a number below `0.5` is an
error. A processor may signal that error; if it recovers, it must convert the
number to a string as if by the XPath `string()` function and insert that
string into the result tree.

FastXSLT's bounded XSLT 1.0 compatibility path follows that recovery. The
archival `0` result does not represent the required recovery and therefore is
not a usable semantic oracle for this case.

## Disposition

`Microsoft/Number__91022#1` now carries an exact
`unusable-reference-result` disposition. It receives no pass credit. The
classification removes one explained archival disagreement from the visible
engine-mismatch count without changing execution or the strict exact-match
lower bound.

After the disposition:

- exact expected-result matches remain `2,221 / 3,173 = 70.00%`;
- initialized cases remain 2,431;
- successful executions remain 2,383;
- visible XML mismatches fall from nine to eight; and
- unusable archival reference results rise from 54 to 55.

The adjacent `Microsoft/Number__84700#1` locale-specific numbering difference
is separately classified as an XSLT 1.0 discretionary-policy exclusion. The
Recommendation permits decimal fallback when a processor does not support the
requested numbering sequence. With both ledger refinements applied, visible
XML mismatches are seven and discretionary-policy exclusions are three.

The bare `Number__91022` entry in the suite's `doubts.xml` does not contain
substantive gray-area metadata and therefore is not counted as a
doubt-annotated mismatch.

## Boundaries

- No corpus bytes or expected results were edited.
- No pass credit was manufactured from an exclusion.
- Modern numbering behavior is unchanged.
- Other values below `0.5` retain the same standards-defined XSLT 1.0 recovery.
- This disposition does not select a general policy for every historical
  processor disagreement.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Number__91022#1'
./scripts/verify.ps1
```

## Standards source

- [W3C XSLT 1.0 errata, E24](https://www.w3.org/1999/11/REC-xslt-19991116-errata/)
