# OASIS XSLT 1.0 Windows-1252 Reference Overlay

Date: 2026-09-28  
Status: Exact four-case local harness evidence

## Question

Can the four remaining non-UTF archival reference files be compared without a
global permissive decoding fallback?

## Method

Byte inspection identified Windows-1252 octets in exactly four reference
files even though their stylesheet/default output metadata does not select that
encoding. The local harness applies a case-identity overlay only to:

- `Output__84374`;
- `Output__84428`;
- `Output__84429`; and
- `Sorting__77977`.

The shared decoder remains strict for every other case. Upstream bytes are not
modified.

## Result

All four references become deterministically readable and all four compare
unequal to FastXSLT's result. They move from comparator gaps to visible
mismatches; none receives pass credit.

Comparator gaps fall from 7 to 3 and visible mismatches rise from 18 to 22.
The strict lower bound remains **2,256 / 3,173 (71.10%)**. Lifecycle,
expected-error, policy, and unusable-reference counters are unchanged.

## Boundaries

- This is test-corpus reference metadata, not an engine output-encoding
  fallback.
- The overlay is exact by case identity and cannot affect future or unrelated
  reference files.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features oasis_archival_reference_encoding_override_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
```
