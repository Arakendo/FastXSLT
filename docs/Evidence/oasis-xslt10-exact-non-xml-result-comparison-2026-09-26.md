# OASIS XSLT 1.0 Exact Non-XML Result Comparison

- Date: 2026-09-26
- Status: Verified comparator tranche
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The archival catalog labels every case in this tranche with `compare="XML"`,
but the selected result methods deliberately produce text or HTML payloads
that are not XML documents or XML fragments. The former comparator therefore
left byte-identical actual and reference payloads unsupported after both XML
parse attempts failed.

An exact normalized lexical comparison is stronger than XML-semantic
equivalence for these payloads. The comparator may therefore accept a result
when the decoded actual and reference content are identical after the same XML
source-line-ending normalization, declaration removal, and outer trimming
already applied by the comparison lane. Unequal non-XML payloads remain
unsupported or unequal; the fallback does not repair, tokenize, or otherwise
interpret malformed markup.

## Admitted identities

Six unchanged cases become exact:

- `Microsoft/Miscellaneous__84427#1`, whose inferred HTML result contains the
  HTML void element spelling `<BR>`;
- `Microsoft/Output__84011#1`, `Output__84016#1`, `Output__84017#1`, and
  `Output__84018#1`, whose text output intentionally contains markup-shaped
  character data; and
- `Microsoft/Output_HtmlOutputWithAmpersandCurlyBracket#1`, whose historical
  HTML reference intentionally contains an unescaped ampersand expression and
  HTML void-element syntax.

The measurement reports these separately as
`exact-normalized-non-xml-comparison-pass` identities. They remain included in
the aggregate exact expected-result count, but cannot be mistaken for XML-tree
comparisons.

## Measurement effect

The conserved sweep remains at 2,342 initialized cases and 2,291 successful
executions. Exact expected-result matches rise from 2,126 to 2,132 out of 3,173
(67.19%), XML comparator gaps fall from 77 to 71, and visible XML mismatches
remain zero. Exactly six passes use this bounded lexical mode.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_xml_comparator_accepts_exact_normalized_non_xml_payloads_only
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
