# OASIS XSLT 1.0 UTF-16 Declaration Transcode -- 2026-09-28

Date: 2026-09-28  
Status: Verified parser and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The private XML adapter already transcoded BOM-selected UTF-16LE and UTF-16BE
into UTF-8 while retaining an original-byte offset map. A document whose XML
declaration explicitly retained `encoding="UTF-16"` nevertheless failed after
transcoding: the byte parser saw that declaration and reselected UTF-16 for the
already UTF-8 buffer.

This was an adapter defect, not malformed corpus input and not a reason to
weaken encoding validation.

## Change

The original-byte lane continues to let the XML reader detect its declared
encoding. The already-transcoded UTF-16 lane now constructs the reader from a
validated Rust string, which fixes the downstream decoder to UTF-8 while the
existing offset map continues to report locations in the immutable admitted
bytes.

The XML declaration is not rewritten, the admitted resource is not mutated,
and truncated code units and invalid surrogate pairs remain errors. A focused
regression now includes a non-ASCII UTF-16 document with an explicit
`encoding="UTF-16"` declaration in both byte orders.

Input preparation and original-offset mapping now live in a private XML-owned
module rather than inside the event loop. This is a responsibility extraction,
not a generalized encoding-provider API; the parser continues to own the only
consumer.

## Corpus result

Unchanged Lotus `output_output80#1` now initializes, executes, and compares
exactly. No other counter changes.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,439 | 2,440 | +1 |
| Executed successfully | 2,391 | 2,392 | +1 |
| Initialization failures | 731 | 730 | -1 |
| Execution failures | 48 | 48 | 0 |
| Exact expected-result matches | 2,228 | 2,229 | +1 |
| Comparator gaps | 54 | 54 | 0 |

The conservative all-catalog exact-match ratio is now
`2,229 / 3,173 = 70.25%`. Expected-error credit remains 423 / 431 and visible
ordinary XML mismatches remain zero.

## Verification

```powershell
cargo test -p fastxslt decodes_bom_selected_utf16_and_preserves_original_byte_offsets
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
