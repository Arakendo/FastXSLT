# OASIS XSLT 1.0 Literal Source-Path AVTs

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can one literal result attribute combine static text with one source location
path while preserving XSLT 1.0 node-set-to-string behavior, work control, and
the existing explicit boundary around the general attribute value template
grammar?

## Changes

- Literal result attributes may retain exactly one typed location path between
  a static prefix and suffix.
- Runtime evaluation uses the existing controlled location-path evaluator from
  the current source focus.
- The selected node set contributes the string value of its first node in
  document order; an empty selection contributes the empty string.
- The compiled path and both literal fragments participate in known retained-
  capacity accounting.
- Malformed braces, multiple dynamic expressions, and expressions outside the
  admitted location-path grammar retain the prior `FXST1031` unsupported
  boundary.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,472 | 1,475 | +3 |
| Executed successfully | 1,309 | 1,311 | +2 |
| Expected-result XML matches | 1,202 | 1,204 | +2 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 164 | +1 |

The unchanged `Microsoft/AVTs__77531#1` and
`Microsoft/AVTs__77575#1` cases now match their expected XML. The latter proves
node-set first-node string conversion through `{*}`. The unchanged
`Lotus/attribvaltemplate_attribvaltemplate05#1` case now executes the mixed
literal/path AVT and reaches the independent bounded HTML-serialization
frontier `FXSR1001`; it remains uncredited.

The strict expected-result lower bound is now 1,204 of 2,742 standard-operation
cases (43.91%) and 1,204 of all 3,173 catalog cases (37.95%). This is local
compatibility evidence, not a conformance claim.

## Boundaries

This tranche does not admit multiple expressions in one AVT, general XPath
expressions, variable/path composition, dynamic QNames or namespaces, temporary-
tree focus, or broader HTML serialization. Those remain independently visible
frontiers.

## Verification

- A focused compiler test proves the bounded mixed representation.
- A focused runtime test proves prefix/suffix composition, first-node
  conversion, and empty-selection conversion.
- The complete local OASIS measurement proves two new exact results, no new
  comparison mismatch, and the explicit later serializer boundary.
- No upstream corpus byte was edited.
