# OASIS XSLT 1.0 HTML URI-Attribute Escaping

Date: 2026-09-17  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the shared HTML serializer correctly distinguish URI-valued HTML
attributes and serialize the unchanged quote/apostrophe case without weakening
expected-result comparison?

## Change

- URI recognition now uses the standard HTML element/attribute pairs instead
  of treating an unnamespaced `href` on every HTML/XHTML element as URI-valued.
- HTML vocabulary matching is ASCII-case-insensitive while both the element and
  attribute remain unprefixed.
- The URI path retains NFC normalization, uppercase UTF-8 percent encoding for
  non-ASCII characters, and existing ASCII percent sequences.
- A double quote is emitted as `%22`; an apostrophe remains literal.
- Ordinary non-URI attributes continue through normal attribute escaping.

The focused runtime case covers case-varied `A`/`HREF`, an embedded double
quote, an apostrophe, and non-ASCII UTF-8 percent encoding.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,939 | 1,939 | 0 |
| Executed successfully | 1,844 | 1,844 | 0 |
| Execution failures | 95 | 95 | 0 |
| Exact XML-semantic expected-result matches | 1,564 | 1,565 | +1 |
| XML comparison mismatches | 209 | 208 | -1 |
| XML comparator unsupported | 64 | 64 | 0 |

The unchanged `Lotus/output_output70#1` case is the sole disposition change.
Its two quote-valued `A/@href` attributes now serialize as `%22`, while its two
apostrophe-valued attributes remain literal, matching the archived expected
result exactly.

## Boundaries

- This is HTML serialization behavior, not URI resolution or resource
  authority.
- No corpus bytes or expected results were changed.
- The exact lower bound rises only by the independently compared case; other
  HTML mismatches and comparator gaps remain visible.
- This evidence does not make the concrete serializer representation public.

## Verification

- The focused runtime serializer test passes.
- The complete local OASIS measurement conserves all 3,173 cases and reports
  1,565 exact matches.
- The workspace verification gates pass.
