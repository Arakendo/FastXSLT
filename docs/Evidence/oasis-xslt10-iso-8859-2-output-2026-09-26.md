# OASIS XSLT 1.0 ISO-8859-2 Output

- Date: 2026-09-26
- Status: Verified focused and conserved-corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged Microsoft case `Output__78221` requests XML serialization using
ISO-8859-2. FastXSLT previously returned structured unsupported `SESU0007`, even
though ISO-8859-2 is a deterministic single-byte encoding and the bounded byte
serialization lane already owned analogous US-ASCII and ISO-8859-1 behavior.

## Implemented slice

The private byte serializer now supports the complete ISO-8859-2 repertoire for
XML, HTML, and text output. The implementation:

- uses a fixed reviewed 96-code-point upper-half table rather than a host
  code-page service;
- emits representable markup and character data directly;
- emits numeric character references for unrepresentable XML character data;
- rejects unrepresentable text/HTML content and unrepresentable markup,
  comment, processing-instruction, or declaration content explicitly;
- retains the existing byte limit, serialized-byte charging, no-BOM rule, and
  semantic-result/physical-serialization boundary; and
- adds focused round-trip and unrepresentable-character tests.

No dependency, ambient locale, host encoding registry, or unsafe code was
introduced. Other legacy encodings remain explicit `SESU0007` boundaries.

## Archival reference finding

The immutable reference result for `Output__78221` is not ISO-8859-2 despite
the stylesheet's declared encoding. It uses Windows-1250 byte assignments; for
example, `š` appears as byte `0x9A`, while ISO-8859-2 assigns `š` to `0xB9`.
Decoding the archive as ISO-8859-2 therefore produces C1 control characters and
cannot serve as an exact standards oracle.

The first-party overlay now classifies this exact case as an unusable archival
reference result. It does not rewrite the upstream bytes or credit an exact
pass. FastXSLT's standards-correct output remains independently covered by the
focused encoding tests.

## Measurement result

`Output__78221` moves from unsupported execution to successful execution. The
conserved sweep remains **2,125 / 3,173 exact matches (66.97%)** and 69 visible
mismatches, while successful execution rises from 2,290 to **2,291** and
execution failures fall from 52 to **51**. Explicit unusable-reference-result
exclusions rise from three to **four**. Initialization remains 2,342 cases and
execution panics remain zero.

This is compatibility and implementation evidence, not an XSLT 1.0 conformance
claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features iso_8859_2
./scripts/measure-oasis-xslt10.ps1
```
