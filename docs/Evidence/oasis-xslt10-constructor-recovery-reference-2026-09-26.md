# OASIS XSLT 1.0 Constructor Recovery Reference

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Microsoft `Elements__78362` places computed and literal element constructors
inside both `xsl:comment` and `xsl:processing-instruction`. Under XSLT 1.0,
creating non-text nodes in those constructors is an error. A processor that
does not signal the error recovers by ignoring offending nodes and their
content.

FastXSLT performs bounded recovery and ignores the non-text nodes. The
catalog-selected output requires one particular whitespace result around those
discarded nodes plus one historical indentation/empty-element layout. Those
details are not a stable semantic oracle for an error-recovery case.

A candidate change that stopped treating stylesheet comments/PIs as
transparent text-run boundaries reduced part of this difference but regressed
three previously exact cases (`axes_axes116` twice and
`whitespace_whitespace21`). The candidate was rejected and fully rolled back;
no shared constructor rule was weakened to imitate this reference.

## Disposition

The identity receives the exact `unusable-reference-result-excluded`
disposition. It remains visible in the conserved 3,173-case denominator and
receives no pass credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,126 exact XML comparisons out of 3,173 (67.00%). Visible XML
mismatches fall from one to zero, and unusable-reference-result exclusions rise
from 45 to 46. This means every successfully executed XML comparison now has an
exact pass or a named non-pass disposition; it is not a broad conformance
claim.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
