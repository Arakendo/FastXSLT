# OASIS XSLT 1.0 Declared Legacy Input Encoding

- Date: 2026-09-26
- Status: Verified private parser and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related reviews: AR-0008 and AR-0019

## Pressure

The conserved OASIS sweep classified several otherwise ordinary stylesheets or
sources as `FXXM0002` because their XML declarations selected a legacy
single-byte encoding. The private `quick-xml` adapter previously interpreted
names, namespace declarations, attribute values, processing instructions, and
text as UTF-8 even when the reader had recognized another encoding.

This was parser-boundary pressure rather than an XSLT compatibility rule. The
engine still needs decoded XML characters while diagnostics and source spans
remain anchored to the admitted resource's original bytes.

## Bounded experiment

The private workbench parser enables `quick-xml` 0.40.1's `encoding` feature and
routes every parser-owned lexical decode through the reader's decoder. Parsing
still consumes the original admitted byte slice, so byte offsets remain stable
for ASCII-compatible encodings. A focused Windows-1252 test verifies a
non-ASCII element name, attribute value, text value, and the unchanged raw
start-tag byte span.

The exact dependency delta is:

- `encoding_rs` 0.8.35, optional and enabled only by FastXSLT's private
  `workbench` feature;
- license expression `(Apache-2.0 OR MIT) AND BSD-3-Clause`;
- declared Rust version 1.36;
- default crate features only; and
- transitive `cfg-if` 1.0.5.

`encoding_rs` contains transitive unsafe implementation code. No first-party
unsafe surface was added, and this evidence does not complete AR-0008's
transitive-unsafe or production dependency admission review. `cargo-audit` was
not installed in the measurement environment, so no vulnerability-audit claim
is made.

UTF-16 remains deliberately unsupported by this slice. Pretranscoding it into
another byte buffer would invalidate direct source offsets unless FastXSLT also
defines and verifies a provenance-preserving offset map.

## Corpus result

Three unchanged cases become exact expected-result matches:

- `Lotus/output_output32`;
- `Microsoft/Sorting__91704`; and
- `Microsoft/Sorting__91705`.

Four more cases initialize. Two Microsoft include cases execute with correct
Japanese text, but their archival expected results replace that text with
literal `???`; `Include__77515` and `Include__77736` therefore receive explicit
unusable-reference-result dispositions rather than false pass credit.
`Lotus/attribvaltemplate_attribvaltemplate08` remains an HTML comparator gap.

The newly admitted `Microsoft/BVTs_bvt067` also exposed an independent output
method defect: XML-only standalone, doctype, and single-document-element
preconditions were being applied to HTML. The shared serializer now limits
those preconditions to XML-like output methods. The unchanged case executes and
remains visibly uncredited pending a semantic HTML comparator.

The conserved sweep changes as follows:

- exact matches rise from 2,155 to **2,158 / 3,173 (68.01%)**;
- initialized cases rise from 2,364 to **2,371**;
- successful executions rise from 2,315 to **2,322**;
- initialization failures fall from 806 to **799**;
- execution failures remain **49**;
- unusable-reference-result exclusions rise from 46 to **48**;
- comparator gaps rise from 67 to **69** because valid HTML cases now reach
  comparison; and
- the two doubt-annotated numeric-rendering mismatches remain visible.

Expected-error credit remains 423 / 431. Seven exact normalized non-XML and
three XML-wrapped text comparisons remain separately identified.

## Boundaries

This tranche establishes only a private parser experiment for XML declarations
supported by the selected decoder, with focused evidence for ASCII-compatible
single-byte input. It does not establish:

- complete XML encoding support;
- UTF-16 or offset-remapping behavior;
- `quick-xml` production admission;
- a public list of supported encodings;
- XML conformance; or
- a completed dependency vulnerability or transitive-unsafe review.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
