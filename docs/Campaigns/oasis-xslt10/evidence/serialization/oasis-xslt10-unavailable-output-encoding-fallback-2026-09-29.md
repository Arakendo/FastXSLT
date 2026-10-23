# OASIS XSLT 1.0 Unavailable Output-Encoding Fallback

- Date: 2026-09-29
- Status: Verified private serializer and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The unchanged `Lotus/output_output77#1` stylesheet requests the deliberately
unavailable encoding label `Big-Deal`, while its expected result is UTF-8 with
an `encoding="UTF-8"` declaration. FastXSLT previously retained the requested
preference and returned `SESU0007` from the private byte serializer.

XSLT 1.0 section 16.1 permits a processor that does not support a requested
encoding other than UTF-8 or UTF-16 either to signal an error or, if it does
not signal an error, to use UTF-8 or UTF-16. The corresponding modern
serialization rule requires `SESU0007` for an unsupported requested encoding.

## Bounded implementation

The compiled output settings continue to retain the stylesheet's exact
requested encoding. The private byte-result entry point now also receives the
compiled stylesheet language version. For an exact-version-1.0 stylesheet
only, an encoding outside the currently admitted byte encoders selects UTF-8
at physical serialization time. The emitted XML declaration and bytes both
truthfully identify UTF-8.

The same unavailable label under an XSLT 3.0 stylesheet still returns
`SESU0007`. Existing UTF-8, UTF-16, US-ASCII, ISO-8859-1, ISO-8859-2,
Shift_JIS, Big5, and ISO-2022-JP byte paths are unchanged. Compilation does not
rewrite the declared preference, and no ambient platform encoding provider is
consulted.

## Corpus result

`Lotus/output_output77#1` now executes and compares exactly after the existing
XML comparison normalizes insignificant result-file layout. The unavailable
EBCDIC case `Lotus/output_output22#1` also executes using the standards-
permitted UTF-8 fallback, but its immutable reference contains EBCDIC bytes.
It therefore remains visibly uncredited as the sole
`expected-not-utf8-or-utf16` comparison boundary rather than disappearing
behind an engine failure.

The conserved sweep changes as follows:

- exact matches rise from 2,278 to **2,279 / 3,173 (71.83%)**;
- successful executions rise from 2,409 to **2,411**;
- execution failures fall from 50 to **48**;
- initialized cases remain **2,459**;
- initialization failures remain **711**;
- visible mismatches and comparator gaps remain **zero**; and
- expected-error credit remains **423 / 431**.

## Boundaries

This evidence does not establish EBCDIC encoding, a public encoding registry,
host encoding-provider access, warning delivery for fallback selection, or a
modern-profile recovery from `SESU0007`. It selects one XSLT 1.0-permitted
physical fallback at the serializer boundary and leaves semantic result
construction unchanged.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_byte_transform_falls_back_to_utf8_for_an_unavailable_encoding
cargo test -p fastxslt --all-features modern_byte_transform_rejects_the_same_unavailable_encoding
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/output_output77#1'
./scripts/verify.ps1
```
