# OASIS XSLT 1.0 HTML Boolean-Attribute Minimization

Date: 2026-09-17  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Change

Legacy/HTML5 serialization now emits the XSLT 1.0 boolean-attribute set in
minimized form when an unnamespaced attribute's value equals its name without
ASCII case sensitivity. Namespaced attributes and values that do not equal the
attribute name retain ordinary quoted serialization.

The admitted set is `checked`, `compact`, `declare`, `defer`, `disabled`,
`ismap`, `multiple`, `nohref`, `noresize`, `noshade`, `nowrap`, `readonly`, and
`selected`.

## Corpus result

The conserved census remains 1,564 exact XML-semantic matches and 1,844
successful executions. The unchanged `Lotus/attribset_attribset17#1` result now
emits `CHECKED` rather than `CHECKED="CHECKED"`, matching the expected HTML
boolean form. Because minimized HTML attributes are not XML syntax, the current
XML-semantic comparator now classifies the actual result as non-parseable rather
than classifying only the expected result that way:

- actual-not-parseable comparator cases: 39 to 40;
- expected-not-parseable comparator cases: 21 to 20; and
- total comparator-unsupported cases: unchanged at 64.

No pass is claimed until an HTML-capable comparison authority can assess the
complete expected result without weakening XML comparison.

## Verification

- A focused general-HTML serializer test covers case-insensitive value matching,
  authored attribute-name spelling, and void-element composition.
- The complete local OASIS measurement conserves all 3,173 catalog cases.
- No upstream corpus byte or expected result was changed.

