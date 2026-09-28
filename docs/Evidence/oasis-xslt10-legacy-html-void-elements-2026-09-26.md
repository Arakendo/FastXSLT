# OASIS XSLT 1.0 Legacy HTML Void Elements

- Date: 2026-09-26
- Status: Verified serializer and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The HTML serializer omitted end tags for the contemporary HTML void-element
set, but its list did not include XSLT 1.0's legacy HTML names `basefont`,
`frame`, and `isindex`. It therefore emitted closing tags that the XSLT 1.0
HTML output method says must not be written.

## Implementation

The private HTML void-name predicate now includes those three legacy names.
Matching remains ASCII case-insensitive, limited to no-namespace result
elements, and active only for the HTML output method. XML and XHTML
serialization retain their existing rules.

The focused HTML serializer test covers all three names alongside uppercase
HTML/script handling.

## Corpus result

Unchanged `Lotus/output_output33` now serializes exactly as the archival
reference and becomes the eighth separately reported exact normalized non-XML
comparison pass. The broad Microsoft `BVTs_bvt067` stress case also receives
correct legacy void-element serialization, but remains visibly uncredited
because independent DOCTYPE, URI-attribute, namespace, and Content-Type
differences remain.

The conserved sweep changes to:

- exact matches: **2,172 / 3,173 (68.45%)**;
- initialized: **2,371**;
- executed successfully: **2,326**;
- execution failures: **45**;
- visible mismatches: **9**, including two doubt-annotated numeric cases;
- comparator gaps: **52**; and
- expected-error credit: **423 / 431**.

## Boundaries

This tranche does not select an HTML version globally, admit namespace-bearing
elements as HTML void elements, create a semantic HTML comparator, or credit
the remaining broad BVT mismatch.

## Verification

```powershell
cargo test -p fastxslt --all-features legacy_html_serialization_recognizes_uppercase_script_and_void_elements
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
