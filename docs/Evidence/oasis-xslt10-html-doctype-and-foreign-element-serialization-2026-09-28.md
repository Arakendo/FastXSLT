# OASIS XSLT 1.0 HTML DOCTYPE and Foreign-Element Serialization

Date: 2026-09-28  
Status: Verified implementation evidence; residual corpus case remains visible  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged `Microsoft/BVTs_bvt067#1` case exposed two independent defects in
the legacy HTML serializer:

1. a configured HTML DOCTYPE used the first result element's name instead of
   the required `html` name; and
2. an attribute whose value equaled its name was minimized on a namespaced
   foreign element merely because the local element and attribute names looked
   like HTML.

## Change

Legacy HTML serialization now:

- emits `html` as the DOCTYPE name independently of the first result element;
  and
- applies HTML boolean-attribute minimization only when the owning element and
  the attribute are both unnamespaced; and
- terminates processing instructions with `>` rather than the XML `?>`
  delimiter.

The semantic result tree is unchanged. XML and XHTML serialization retain
their established behavior, and the existing output-byte budget still covers
the emitted declaration and attributes.

Focused regressions exercise a non-`html` first result element and a
namespaced foreign `input` carrying `checked="checked"`.

## Corpus result

The first two defects are corrected in the unchanged BVT output. The case still
contains several independent, unimplemented legacy HTML behaviors, including
URI-attribute handling, content-type metadata, foreign empty-element syntax,
raw-text escaping, and layout differences. It therefore remains a visible
comparator gap and receives no pass credit.

The processing-instruction correction was exercised independently by unchanged
Lotus `output_output36`. After the bounded HTML comparator was introduced, the
case became a visible layout mismatch rather than an unparseable result. It
also receives no pass credit.

This tranche changes no catalog pass counter by itself. The gap is deliberately
not hidden behind a special-case comparator or an inferred pass.

## Boundaries

- This does not claim general HTML 4 serialization conformance.
- Namespaced elements continue to serialize by the XML-compatible foreign-
  element rules within the HTML method.
- No corpus reference bytes or comparison policy were changed.

## Verification

```powershell
cargo test -p fastxslt legacy_html_serialization_keeps_boolean_attributes_quoted_on_foreign_elements
cargo test -p fastxslt html_output_uses_html_as_the_doctype_name_for_a_non_html_first_element
cargo test -p fastxslt processing_instruction_serializes_as_markup_but_not_as_text_value
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt067#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/output_output36#1'
./scripts/verify.ps1
```

## Standards source

- [W3C XSLT 1.0 section 16.2](https://www.w3.org/TR/1999/REC-xslt-19991116#output)
