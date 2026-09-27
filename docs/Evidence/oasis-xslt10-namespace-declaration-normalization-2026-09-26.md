# OASIS XSLT 1.0 Namespace-Declaration Normalization

- Date: 2026-09-26
- Status: Implemented and verified
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0008, AR-0019

## Pressure

Unchanged Microsoft `Elements__78364` constructs an element whose namespace URI
comes from an `xsl:element namespace` attribute containing `&amp;` and a physical
CRLF pair. The stylesheet attribute was decoded and normalized correctly, so
FastXSLT serialized the resulting namespace URI with `&` and a space. The XML
comparison parser, however, resolved the expected element name through
`quick-xml`'s raw namespace resolver bytes. That left `&amp;` and CRLF spelling in
the expanded name and created a false semantic mismatch.

## Repair

The parser adapter now normalizes a resolved namespace declaration before it
becomes an engine-owned expanded name:

1. literal XML tabs and line endings are normalized to spaces, with CRLF
   treated as one line ending; and
2. predefined and numeric character/entity references are then decoded.

Doing literal whitespace normalization before reference decoding preserves the
XML distinction between a physical line break and a character reference. The
same rule applies to root, descendant, attribute, and end-name namespace
resolution through the existing adapter function; no XDM or serializer special
case was added.

## Verification

A focused parser test resolves both a root and inherited child name under
`xmlns="urn:a&amp;b&#x0D;&#x0A;c"`-shaped pressure expressed with a physical CRLF,
asserting the normalized expanded namespace `urn:a&b c`. The unchanged OASIS
case then compares exactly.

The sweep advances from 2,125 to **2,126 / 3,173 exact XML comparisons
(67.00%)**. Initialized and successfully executed totals remain 2,342 and
2,291. Visible XML mismatches fall from 14 to 13, and no exclusion is added.

```powershell
cargo test -p fastxslt --all-features normalizes_namespace_declaration_values_before_name_resolution
./scripts/measure-oasis-xslt10.ps1 -TraceCase Elements__78364
./scripts/verify.ps1
```
