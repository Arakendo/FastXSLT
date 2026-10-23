# OASIS XSLT 1.0 Residual HTML Comparison

Date: 2026-09-28  
Status: Local comparator evidence

## Question

Can the test-only OASIS comparator distinguish more exact HTML results from
real semantic differences without changing engine serialization or weakening
the strict exact-match lower bound?

## Method

- Before applying HTML lexical normalization, compare two already XML-readable
  documents by expanded name. Return early only for equality; an unequal raw
  comparison still proceeds through the existing HTML rules.
- Decode only the four named HTML references present in the residual archival
  cases: `nbsp`, `copy`, `Egrave`, and `oacute`.
- Keep this logic private to the local measurement harness. Engine result trees,
  serializer behavior, and upstream expected bytes remain unchanged.
- Re-run all 3,173 catalog cases after focused comparator tests.

## Result

`Lotus/output_output63#1` is already XML-readable on both sides. Comparing it
before HTML void-element rewriting preserves its foreign namespace and makes
the explicit-start/end and empty-element spellings exactly equivalent.

The bounded named-reference decoding partitions three former parser gaps:

- `Lotus/attribvaltemplate_attribvaltemplate08#1` becomes an exact semantic
  match because `&oacute;` and the literal Unicode character are equal;
- `Lotus/copy_copy38#1` becomes a visible mismatch because the expected result
  contains non-breaking spaces absent from the actual result; and
- `Lotus/output_output04#1` becomes a visible mismatch for the same missing
  non-breaking-space reason, even though its other named references are equal
  to the actual Unicode characters.

The strict lower bound rises from **2,254 / 3,173 (71.04%)** to
**2,256 / 3,173 (71.10%)**. Comparator gaps fall from 12 to 8 and visible
mismatches rise from 16 to 18. Initialization, execution, expected-error, and
policy counters are unchanged.

## Boundaries

- This is not a general HTML parser or a public comparison API.
- Unknown named references remain unsupported rather than being guessed.
- A raw XML mismatch is not accepted; it merely falls through to the bounded
  HTML normalization path.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features oasis_html_comparator
cargo test -p fastxslt --all-features oasis_html_comparator_preserves_xml_readable_foreign_element_names
./scripts/measure-oasis-xslt10.ps1
```
