# OASIS XSLT 1.0 HTML Ordinary Attribute Escaping

Date: 2026-09-17  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the HTML serializer stop applying XML attribute escaping to ordinary HTML
attributes without changing XML/XHTML behavior or weakening result comparison?

## Change

Ordinary HTML attribute serialization now:

- preserves `<` and `>` characters;
- preserves tab, line feed, and carriage return characters;
- escapes ampersands and double quotes;
- emits C1 controls as hexadecimal numeric character references; and
- continues to apply character maps and the selected normalization form.

URI-valued HTML attributes retain their separate URI-escaping path. XML and
XHTML output retain XML-compatible attribute escaping.

## Corpus result

All measured counts remain conserved:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Executed successfully | 1,844 | 1,844 | 0 |
| Exact XML-semantic expected-result matches | 1,565 | 1,565 | 0 |
| XML comparison mismatches | 208 | 208 | 0 |
| XML comparator unsupported | 64 | 64 | 0 |

The unchanged
`Microsoft/Output_HtmlOutputWithLessThanInAttribute#1` case now emits
`bar="<bar>"` rather than XML entity references. The unchanged
`Microsoft/Output_EntityRefInAttribHtml#1` case now emits the result-tree line
feed literally rather than as `&#xA;`. Both retain independent expected-output
pretty-print or newline differences and therefore remain visible mismatches.
Neither is credited as a pass.

## Boundaries

- This does not select a general HTML pretty-printing policy.
- The archival expected result's CRLF spelling is not copied into semantic
  result content or made platform-dependent.
- The special legacy `&{` compatibility rule remains separate future work.
- No corpus bytes or expected results were changed.

## Verification

- Focused runtime coverage distinguishes HTML ordinary-attribute behavior from
  the existing XHTML controls.
- The complete local OASIS measurement conserves all 3,173 cases and the 1,565
  exact lower bound.
- The workspace verification gates pass.
