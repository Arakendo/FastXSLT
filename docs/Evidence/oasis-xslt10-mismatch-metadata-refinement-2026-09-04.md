# OASIS XSLT 1.0 Mismatch Metadata Refinement

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input | 468 definite passes; 29 XML comparison mismatches |
| Result | 2 mismatches carry substantive doubts metadata; 27 do not |
| Disposition | Reporting refinement; no case reclassified |

## Observation

The suite's `doubts.xml` lists many case identities with empty entries and a
smaller set with actual child metadata. The local runner already counted only
the latter as doubt-annotated identities. It now applies that same definition
to executing XML mismatches and emits the exact matching identities.

At the 468-pass checkpoint, only these two raw mismatches carry substantive
doubts metadata:

- `Microsoft/Output__77927#1`
- `Microsoft/Output__77928#1`

Both remain mismatches. The report does not convert doubt metadata into an
exclusion or a pass, and it does not infer that the other 27 results are engine
defects. Individual review must still separate result-tree semantics,
serialization freedom, expected-file conventions, and comparator limitations.

## Why this matters

An identity's mere presence as an empty element in `doubts.xml` is not evidence
of a recorded dispute. Conversely, silently removing the two annotated cases
would alter the denominator before FastXSLT has selected a compatibility
profile. The refined counter preserves both facts without denominator sorcery.

## Boundary

This is not the versioned final-disposition overlay required by AR-0019. It
adds reproducible metadata to the exploratory local sweep and leaves all 29
raw mismatches visible.
