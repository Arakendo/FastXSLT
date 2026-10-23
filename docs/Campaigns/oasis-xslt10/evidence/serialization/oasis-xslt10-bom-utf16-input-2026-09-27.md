# OASIS XSLT 1.0 BOM-Selected UTF-16 Input -- 2026-09-27

Date: 2026-09-27  
Status: Verified parser and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The private XML adapter accepted UTF-8 and declared ASCII-compatible legacy
encodings, but rejected BOM-marked UTF-16 stylesheet bytes before XSLT
compilation. The standard-operation frontier included a concentrated Microsoft
numbering family, so this was XML input pressure rather than evidence for a new
numbering shortcut.

## Change

The adapter now recognizes UTF-16LE and UTF-16BE byte-order marks, decodes the
code units strictly into its private parser feed, and maps every parser byte
boundary back to the original admitted byte offset. Unpaired surrogates and
truncated code units remain malformed input. The immutable admitted resource
bytes, resource identity, DTD prohibition, event/depth limits, and owned XDM
boundary are unchanged.

This is still a private parser experiment under AR-0008. It does not select a
public encoding profile or admit `quick-xml` as a stable production dependency.

## Corpus result

Twenty-two standard-operation cases leave the generic stylesheet-XML frontier.
Eleven compile completely, nine execute, and these six unchanged cases compare
exactly:

- `Microsoft/Number__84679#1`
- `Microsoft/Number__84680#1`
- `Microsoft/Number__84681#1`
- `Microsoft/Number__84682#1`
- `Microsoft/Number__84684#1`
- `Microsoft/Number__84698#1`

The remaining observations stay explicit: fourteen cases expose unsupported
numbering formats, two reach existing runtime formatting limits, and three
execute to visible mismatches. Two of those mismatches are already annotated
with upstream doubt metadata.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,371 | 2,382 | +11 |
| Executed successfully | 2,326 | 2,335 | +9 |
| Initialization failures | 799 | 788 | -11 |
| Execution failures | 45 | 47 | +2 |
| Exact expected-result matches | 2,172 | 2,178 | +6 |
| Visible mismatches | 3 | 6 | +3 |

The conservative all-catalog exact-match ratio is now
`2,178 / 3,173 = 68.64%`. Expected-error credit remains 423 / 431 and comparator
gaps remain 52.

## Verification

- Focused tests cover UTF-16LE and UTF-16BE, non-ASCII names and text,
  original-byte spans, an incomplete code unit, and an unpaired surrogate.
- The unchanged 3,173-case catalog sweep produced the counters above.
- The ordinary workspace verification gate passes.

