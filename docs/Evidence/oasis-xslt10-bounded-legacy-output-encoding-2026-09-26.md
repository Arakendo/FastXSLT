# OASIS XSLT 1.0 Bounded Legacy Output Encoding

- Date: 2026-09-26
- Status: Verified private serializer and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related reviews: AR-0008 and AR-0019

## Pressure

Six otherwise executable Lotus output cases selected encodings outside the
private byte serializer's UTF-8, US-ASCII, and ISO-8859 lanes. Four used
standard encodings already supplied by the optional private `encoding_rs`
dependency: Shift_JIS, Big5, and ISO-2022-JP. The other two selected
`EBCDIC-CP-IT` and the deliberately unknown label `Big-Deal`.

This was a physical serialization boundary. It did not justify changing the
semantic result tree or treating an encoding label as host authority.

## Bounded experiment

The private byte lane recognizes only these additional labels:

- `SHIFT_JIS` and `SHIFT-JIS`;
- `BIG5`; and
- `ISO-2022-JP`.

The complete UTF-8 serialization is encoded through the selected
`encoding_rs` encoder. If any result character is not representable, the lane
rejects the result as `FXSR1006` instead of accepting the encoder's replacement
or numeric-reference fallback. Focused tests cover all three admitted
encodings, declaration bytes, round-trip decoding, and rejection of an emoji
under Shift_JIS.

The existing serializer charges the UTF-8 body before physical encoding and
then charges any positive expansion in the target encoding. It therefore
remains bounded but can conservatively overcharge when the target bytes are
shorter than UTF-8. This tranche does not claim exact target-byte work
accounting.

HTML Content-Type metadata now uses the selected physical encoding rather than
the UTF-8 encoding of the intermediate Rust string. The focused Shift_JIS HTML
case therefore emits `charset=SHIFT_JIS` before the complete result is encoded.
This fixes a real serializer-consistency defect without changing the semantic
result tree or claiming a general HTML comparison model.

The local OASIS comparator gained strict decoding for the same three labels.
Invalid byte sequences remain comparator failures; there is no replacement-
character recovery.

## Corpus result

Four unchanged cases leave `SESU0007` and execute:

- `Lotus/output_output20` (Shift_JIS), exact;
- `Lotus/output_output21` (Big5), exact;
- `Lotus/output_output23` (ISO-2022-JP), exact; and
- `Lotus/output_output73` (Shift_JIS), with truthful Shift_JIS Content-Type
  metadata but still visibly uncredited at the existing semantic
  HTML-comparator boundary because the archival reference differs only in
  HTML name casing and layout.

`Lotus/output_output22` (`EBCDIC-CP-IT`) and `Lotus/output_output77`
(`Big-Deal`) remain explicitly unsupported as `SESU0007`.

The conserved sweep changes as follows:

- exact matches rise from 2,158 to **2,161 / 3,173 (68.11%)**;
- successful executions rise from 2,322 to **2,326**;
- execution failures fall from 49 to **45**;
- initialized cases remain **2,371**;
- comparator gaps rise from 69 to **70** because `output73` now reaches
  comparison; and
- the two doubt-annotated numeric-rendering mismatches remain visible.

Expected-error credit remains 423 / 431. Seven exact normalized non-XML and
three XML-wrapped text comparisons remain separately identified.

## Boundaries

This tranche does not establish:

- a general or public supported-output-encoding registry;
- EBCDIC support or unknown-label fallback;
- character-reference substitution for unrepresentable characters;
- exact target-byte work accounting;
- a semantic HTML comparator;
- production admission of `encoding_rs`; or
- a completed dependency vulnerability or transitive-unsafe review.

The encoding dependency remains optional behind the private `workbench`
feature, with a dev dependency for focused and corpus tests. No first-party
unsafe code was added.

## Verification

```powershell
cargo test -p fastxslt byte_serialization_emits_bounded_legacy_multibyte_encodings --all-features
cargo test -p fastxslt oasis_actual_decoder_admits_selected_bounded_legacy_bytes --all-features
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
