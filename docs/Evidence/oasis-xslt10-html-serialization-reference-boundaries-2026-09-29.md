# OASIS XSLT 1.0 HTML Serialization Reference Boundaries

- Date: 2026-09-29
- Status: Verified implementation and non-pass corpus-classification evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

`Microsoft/BVTs_bvt067#1` was the last successfully executed case for which
the local harness could not produce a comparison disposition. Its HTML result
combines void and foreign elements, Boolean attributes, URI attributes,
script/style content, generated Content-Type metadata, namespaces, and mixed
case.

Making that result XML-readable also exposed engine behavior that had made
`Microsoft/Output__84165#1` compare exactly only by reproducing an over-broad
archival serializer rule.

## Comparator correction

The test-only bounded HTML normalizer now:

- does not treat an unprefixed element with a non-empty default namespace as
  an HTML void element; and
- preserves the case of prefixed foreign element names in both start and end
  tags.

The comparator remains a local measurement tool. It does not modify engine
results or upstream reference bytes. After normalization, `BVTs_bvt067` is a
definite unequal result rather than an unsupported comparison.

## Engine corrections

Legacy HTML serialization now:

- minimizes a Boolean attribute only for an HTML element/attribute pair for
  which that attribute is Boolean;
- leaves `<` unescaped in null-namespace HTML attribute values;
- preserves `&` immediately followed by `{` in those values;
- percent-encodes non-ASCII URI characters without reintroducing XML escaping
  for those HTML characters;
- applies XML attribute escaping to foreign namespaced elements; and
- carries the script/style raw-text context through nested result elements,
  rather than applying it only to direct text children.

Focused regressions retain the admitted Boolean pairs, distinguish
`input checked` from `link checked="checked"`, preserve foreign-element syntax,
and exercise URI and nested raw-text behavior. Output-byte accounting remains
on the existing bounded serializer path.

## Exact archival-reference dispositions

Two references cannot serve as exact normative oracles for the corrected
behavior:

- `Microsoft/Output__84165#1` minimizes every known Boolean attribute on any
  HTML element that has at least one Boolean attribute. For example, it expects
  `checked` to be minimized on `TH`, even though `checked` is not a Boolean
  attribute of `TH`.
- `Microsoft/BVTs_bvt067#1` requests ISO-8859-1 HTML output but expects generated
  Content-Type `META` elements whose `content` value omits the charset. XSLT
  1.0 requires inserted metadata to specify the actual character encoding.

Both identities receive exact `unusable-reference-result-excluded`
dispositions. They receive no pass credit, and their expectations do not weaken
the serializer.

## Measurement result

The complete conserved sweep now reports:

- 3,173 catalog cases;
- 2,440 initialized cases;
- 2,391 successful executions;
- **2,260 / 3,173 exact matches (71.23%)**;
- 70 unusable-reference exclusions;
- zero visible comparison mismatches; and
- zero comparator-unsupported cases.

The exact lower bound decreases by one from 2,261 because the formerly exact
`Output__84165` result depended on the over-broad Boolean minimization rule.
This is an intentional correctness correction, not a coverage regression hidden
by normalization. The ledger still conserves every case and exposes both
uncredited identities.

This is compatibility evidence from a locally acquired archival corpus, not an
XSLT 1.0 conformance claim.

## Verification

```powershell
cargo test -p fastxslt --all-features legacy_html_serialization_accepts_a_general_nested_result_tree
cargo test -p fastxslt --all-features legacy_html_serialization_keeps_boolean_attributes_quoted_on_foreign_elements
cargo test -p fastxslt --all-features legacy_html_serialization_minimizes_only_valid_boolean_element_attribute_pairs
cargo test -p fastxslt --all-features legacy_html_serialization_recognizes_uppercase_script_and_void_elements
cargo test -p fastxslt --all-features preserves_foreign_elements_whose_local_names_look_like_html_void_elements
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```

## Standards source

- [W3C XSLT 1.0 section 16.2, HTML Output Method](https://www.w3.org/TR/1999/REC-xslt-19991116#output)
