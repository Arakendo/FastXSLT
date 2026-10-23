# OASIS XSLT 1.0 attribute text and numeric-sort metadata -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can two bounded syntactic forms move through the shared engine without
claiming semantics that FastXSLT does not implement: explicit `xsl:text`
content inside a computed attribute, and static collation metadata on a
numeric sort key?

## Changes

- A computed attribute may contain exactly one `xsl:text` child. The compiler
  reuses the ordinary character-content and `disable-output-escaping`
  validation and lowers the result to the existing immutable text value.
- A numeric `xsl:sort` may carry static `lang` and `case-order` values. They are
  discarded during compilation because numeric ordering does not use text
  collation.
- Text sorts carrying either property fail explicitly with `FXST1063` rather
  than silently using FastXSLT's current codepoint comparison.
- Dynamic attribute value templates in otherwise ignored numeric-sort metadata
  fail explicitly with `FXST1064`; the engine does not skip their evaluation
  and possible failure by accident.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,562 | 1,566 | +4 |
| Executed successfully | 1,392 | 1,395 | +3 |
| Expected-result XML matches | 1,272 | 1,275 | +3 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 170 | 171 | +1 |

The attribute-text refinement moves six cases beyond `FXST1033`, reducing that
frontier from 27 to 21. Most meet another compilation boundary. Lotus
`output60` reaches the existing bounded HTML-serialization rejection, producing
the one additional execution failure; it remains uncredited. The three exact
new results come from static metadata on numeric sorts.

The strict standard-operation lower bound is now
`1,275 / 2,742 = 46.50%`; the conservative all-catalog ratio is
`1,275 / 3,173 = 40.18%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not implement locale-sensitive collation, case ordering for
text keys, dynamic sort metadata, general sequence construction inside an
attribute, or `disable-output-escaping="yes"`. No corpus bytes or expected
results changed.

## Verification

- Focused runtime tests cover explicit `xsl:text` attribute content and static
  numeric-sort metadata.
- Focused compiler tests preserve explicit text-collation and dynamic-metadata
  failures.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
