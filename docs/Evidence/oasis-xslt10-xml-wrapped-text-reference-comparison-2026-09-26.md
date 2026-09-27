# OASIS XSLT 1.0 XML-Wrapped Text Reference Comparison

- Date: 2026-09-26
- Status: Verified comparator tranche
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Microsoft `Output__84009` and `Output__84014` select the XSLT text output
method, but their catalog entries request XML comparison and their archival
references encode the intended text as an XML-declaration-prefixed fragment.
FastXSLT correctly emits raw text, so treating the actual bytes as XML leaves
both cases at `actual-not-parseable-document-or-fragment` even though the
decoded reference string-value is exact.

## Bounded comparison rule

The OASIS-only comparator may decode an XML-wrapped reference to its string-
value only when all of these conditions hold:

- the compiled stylesheet selected `method="text"`;
- the archival reference begins with an XML declaration;
- the reference parses as the existing bounded comparison fragment; and
- the normalized raw actual text equals the parsed reference string-value.

References without the XML declaration retain the existing corpus comparison
path. This matters because the archive also contains raw text references whose
entity spelling is literal text rather than an XML wrapper. The adapter does
not guess between those two conventions from markup-shaped content alone.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases and 2,291 successful
executions. Exact expected-result matches rise from 2,132 to 2,134 out of 3,173
(67.25%), comparator gaps fall from 71 to 69, and visible XML mismatches remain
zero. The two passes remain separately reported as
`xml-wrapped-text-comparison-pass` identities.

A focused text-output regression also proves that whitespace-only stylesheet
boundaries after copying a temporary tree do not leak into the serialized text
result. This protects the raw-reference convention while the wrapper convention
is handled only by the measurement adapter.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_text_comparator_decodes_an_xml_wrapped_archival_reference
cargo test -p fastxslt --all-features xslt10_text_output_drops_whitespace_only_template_boundaries_after_temporary_copy
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
