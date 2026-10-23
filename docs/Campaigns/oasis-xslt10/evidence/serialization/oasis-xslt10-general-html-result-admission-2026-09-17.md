# OASIS XSLT 1.0 General HTML Result Admission

Date: 2026-09-17  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing legacy/HTML5 serializer accept ordinary semantic result trees
without retaining a growing whitelist of corpus-specific tree shapes?

## Change

- Explicit `method="html"` now accepts general semantic result trees after the
  existing serialization-property and processing-instruction validation.
- An absent output method selects legacy HTML serialization when the first
  significant result node is an unnamespaced `html` element, as required by
  the adaptive XSLT output-method rule.
- The serializer continues to use the existing byte budget, cancellation,
  namespace-scope, URI-attribute, raw-text, void-element, indentation, content-
  type, character-map, and encoding paths.
- The former result-shape validator remains private as a conservative reference
  oracle for the shapes that incrementally established those mechanisms; it is
  no longer a production admission gate.

This admits result-tree breadth. It does not claim that every HTML serialization
rule or every OASIS expected-result comparison is correct.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,939 | 1,939 | 0 |
| Executed successfully | 1,725 | 1,844 | +119 |
| Execution failures | 214 | 95 | -119 |
| `FXSR1001` execution frontier | 119 | 0 | -119 |
| Exact XML-semantic expected-result matches | 1,559 | 1,564 | +5 |
| XML comparison mismatches | 137 | 209 | +72 |
| XML comparator unsupported | 23 | 64 | +41 |
| Expected execution errors observed | 23 | 22 | -1 |
| Expected errors that unexpectedly succeeded | 4 | 5 | +1 |

The disposition conservation is deliberate: the 119 formerly blocked cases
become five exact matches, 72 visible mismatches, 41 visible comparator gaps,
and one visible missing error. The change therefore raises the strict exact
lower bound only to 1,564 while exposing substantially more real serializer and
harness work.

The extra unexpected success is `Microsoft/Errors_err031#1`; general HTML
admission reveals that the current static computed-element validation does not
produce the error expected by that archival case. It remains visible and is not
credited.

## Boundaries

- General result-tree admission is not a claim of complete XSLT 1.0 HTML output
  conformance.
- The corpus runner's XML-semantic comparator cannot yet assess every legal
  HTML serialization, particularly void-element output and expected HTML that
  is not an XML document or fragment.
- Mismatches are not converted into passes merely because an HTML parser might
  accept both byte streams.
- No corpus bytes or expected results were changed.
- The output method remains a serializer concern; semantic result construction
  is unchanged.

## Verification

- Focused runtime tests cover adaptive method selection and a general nested
  HTML tree outside the former whitelist.
- Existing HTML5, raw-text, void-element, content-type, character-map, URI,
  byte-limit, and cancellation tests continue to exercise the shared serializer.
- The complete local OASIS measurement conserves all 3,173 catalog cases and
  exposes every later disposition.

