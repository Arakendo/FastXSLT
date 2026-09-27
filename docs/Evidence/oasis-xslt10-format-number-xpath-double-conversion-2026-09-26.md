# OASIS XSLT 1.0 `format-number()` XPath-Double Conversion

Date: 2026-09-26  
Status: Verified shared-engine evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Does the XSLT 1.0 compatibility path convert the first `format-number()`
argument through XPath 1.0's IEEE-754 `number` type before formatting, while
the modern engine retains its exact-decimal behavior?

## Defect

The formatter parsed the first argument as `f64` for sign, NaN, and infinity
behavior, but its finite formatting path could then return to the original
decimal lexical value. Very large finite values therefore retained precision
that XPath 1.0 had already discarded.

For example, a source-derived 70-digit sequence of nines was formatted as the
same 70-digit exact decimal rather than the double-rounded value `1` followed
by 70 zeroes.

## Change

- The XSLT 1.0 compatibility path now derives finite formatting digits from
  the already converted IEEE-754 value.
- The modern path continues to prefer its exact source-free decimal or
  rational representation.
- NaN, infinity, negative-zero, decimal-format, picture, and charged runtime
  behavior remain on their existing paths.

This is compatibility-profile isolation, not a replacement of the modern
numeric model with XSLT 1.0 doubles.

## Verification

- A focused formatter test proves that identical large input is double-rounded
  in XSLT 1.0 mode and remains exact in modern mode.
- An end-to-end transform proves the rule for a source-attribute variable and
  also retains ordinary `1,234.50` formatting.
- The complete unchanged 3,173-case catalog was rerun with zero panics.

## Corpus effect

The repair corrects the large-number fields exercised inside the remaining
Microsoft `format-number()` mismatch cluster. Those cases also contain
independent legacy output/reference differences and do not yet become exact.
The honest totals therefore remain:

- 2,342 initialized;
- 2,291 executed successfully;
- 2,125 / 3,173 exact matches (66.97%); and
- 64 visible XML mismatches.

No pass or exclusion is awarded by this tranche.
