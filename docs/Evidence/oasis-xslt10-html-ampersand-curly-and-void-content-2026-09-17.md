# OASIS XSLT 1.0 HTML Ampersand-Curly and Void Content

Date: 2026-09-17  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the legacy HTML serializer preserve the XSLT 1.0 `&{` compatibility rule
and omit end tags for HTML void elements without weakening comparison or
changing XML/XHTML output?

## Change

- On the no-character-map HTML path, an ampersand immediately followed by `{`
  remains literal. Other ampersands remain escaped.
- HTML void elements omit their end tags even when the semantic result tree
  supplied child content. Such content is serialized after the start tag.
- XML and XHTML output retain XML-compatible ampersand and end-tag behavior.
- Character-map composition is deliberately unchanged; this slice does not
  infer ordering between the legacy compatibility rule and later-version
  character maps.

Focused runtime coverage combines `&{...}`, an ordinary ampersand, and a void
element with supplied child content.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Executed successfully | 1,844 | 1,844 | 0 |
| Exact XML-semantic expected-result matches | 1,565 | 1,565 | 0 |
| XML comparison mismatches | 208 | 208 | 0 |
| XML comparator unsupported | 64 | 64 | 0 |

`Lotus/output_output37#1` now preserves `&{randomrbg};` but retains independent
DOCTYPE/indentation differences. The unchanged
`Microsoft/Output_HtmlOutputWithAmpersandCurlyBracket#1` output now preserves
`&{foo}` and omits the `area` end tag while retaining its child text, matching
the relevant archived HTML spelling. The strict XML-semantic harness does not
promote either case, because the remaining presentation or non-XML HTML
comparison boundary is still material.

## Boundaries

- No corpus bytes or expected results were changed.
- The serializer does not discard result-tree children merely because the
  element name is void in HTML.
- This does not select a permissive HTML comparator.
- This does not generalize the `&{` rule through character-map replacement
  strings.

## Verification

- Focused runtime serialization passes.
- The complete local OASIS measurement conserves all 3,173 cases and the 1,565
  exact lower bound.
- The workspace verification gates pass.
