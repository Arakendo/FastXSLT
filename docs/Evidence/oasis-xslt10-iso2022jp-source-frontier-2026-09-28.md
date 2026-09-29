# OASIS XSLT 1.0 ISO-2022-JP Source Frontier -- 2026-09-28

Date: 2026-09-28  
Status: Negative boundary evidence; no capability admitted  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Question

Eleven Microsoft output cases share the declared `ISO-2022-JP` principal source
`Output/iso-2022-jp.xml`. Could the XML input adapter explicitly transcode that
stateful encoding to UTF-8, preserve admitted-byte source locations, and move
the cases beyond source initialization?

## Observation

A bounded incremental prototype decoded ordinary shifted Japanese text and
could maintain a monotonic UTF-8-to-original-byte boundary map. The unchanged
archival source nevertheless fails strict `encoding_rs` decoding at original
byte offset 994. At that point the source contains an ASCII reset immediately
followed by another Japanese shift (`ESC ( B ESC $ B`) without intervening
character output.

The strict decoder rejects this redundant state transition. Accepting the
archival source would therefore require a FastXSLT-owned permissive legacy
decoder rule, not merely wiring an already-supported decoder into the parser.
The affected cases then diverge into several output encodings and do not offer
one narrow semantic payoff.

## Disposition

No permissive ISO-2022-JP input behavior is admitted. The experimental decoder
path was removed. Original bytes continue through the existing XML decoder and
the 11 cases remain visible initialization failures.

This is deliberately different from the BOM-selected UTF-16 lane: UTF-16 has a
complete strict transcode with exact original-byte mapping, while this archival
ISO-2022-JP source would require a compatibility policy FastXSLT has not
selected. A future consumer requiring legacy stateful input can reopen the
question with an explicit decoder contract and security/conformance evidence.

## Measurement

The conserved sweep remains unchanged at 2,229 exact matches out of 3,173
catalog cases (70.25%), 2,440 initialized cases, and 2,392 successful
executions.

```powershell
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Output__78222#1'
./scripts/measure-oasis-xslt10.ps1
```
