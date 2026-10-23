# OASIS XSLT 1.0 Text-Sort Policy and BVT Reference

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Five successfully executed sort comparisons remained visible mismatches:

- Lotus `sort_sort08` and `sort_sort27`;
- Microsoft `Sorting__78286` and `Sorting__78291`; and
- Microsoft `BVTs_bvt083`.

The first four use text sorting without selecting a portable collation. Their
archival results order spaces, punctuation, signs, and decimal points according
to historical processor/environment rules that differ from FastXSLT's current
deterministic bounded text ordering. Numeric sorting in the same Microsoft
cases is exact.

`BVTs_bvt083` is different. Its final loop sorts eight `last-name` elements by
the standard expression `self::node()[current() = .]`. FastXSLT evaluates the
non-empty key and produces lexical order. The expected result preserves the
source order exactly, as though this one sort instruction had no effect, even
though earlier sort instructions in the same stylesheet are reflected in the
same expected file.

## Disposition

XSLT 1.0 text sorting does not provide one portable, environment-independent
default ordering for these punctuation and whitespace cases. The four pure
ordering disagreements therefore receive a distinct
`host-collation-policy-excluded` disposition. This is not a pass, and it does
not silently select the archival processor's collation as FastXSLT's public
contract.

`BVTs_bvt083` receives an `unusable-reference-result-excluded` disposition.
Treating its expected source order as authoritative would require suppressing a
standards-defined sort whose non-empty key FastXSLT already evaluates through
the shared `current()` semantics. It is not grouped with the collation cases
because its expected order is the unsorted input, not an alternate observable
text collation.

All five cases remain named in the conserved 3,173-case denominator and receive
no pass credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,125 exact XML comparisons out of 3,173 (66.97%). Visible XML
mismatches fall from 27 to 22. Four host-collation-policy exclusions are added,
and unusable-reference-result exclusions rise from 36 to 37.

## Verification

The measurement adapter has exact bounded-list controls for both dispositions.
The unchanged trace retains the actual and expected ordering, while the existing
sort suite continues to verify stable ties, numeric sorting, and the admitted
text-order implementation.

```powershell
cargo test -p fastxslt --all-features oasis_host_collation_policy_exclusion_is_exact_and_bounded
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
