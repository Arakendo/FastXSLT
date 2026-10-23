# OASIS XSLT 1.0 BVT String-Value Reference

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Microsoft case `BVTs_bvt091` evaluates `xsl:value-of select="."` for source
`story1` and `story2` elements containing text interleaved with descendant
elements. XPath 1.0 defines an element's string-value as the concatenation, in
document order, of all descendant text-node string values. It does not insert
a separator or normalize those text nodes.

FastXSLT returns that exact source-derived value, including the source newline
and tab text between `details`, `stock`, and `date`. The catalog-selected
reference replaces those boundaries with invented single spaces. The
stylesheet contains neither `normalize-space()` nor any other operation that
can produce that transformation.

## Disposition

The case receives the exact `unusable-reference-result-excluded` disposition.
FastXSLT retains the standard XDM/XPath string-value shared with the modern
engine instead of adding a BVT-specific normalization path. The case remains
visible in the conserved 3,173-case denominator and receives no pass credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,126 exact XML comparisons out of 3,173 (67.00%). Visible XML
mismatches fall from 11 to 10, and unusable-reference-result exclusions rise
from 39 to 40.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt091#1'
./scripts/verify.ps1
```
