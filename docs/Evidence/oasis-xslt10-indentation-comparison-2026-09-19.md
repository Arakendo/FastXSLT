# OASIS XSLT 1.0 Indentation-Neutral XML Comparison

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

How many executing XML cases were being reported as mismatches solely because
the comparator treated implementation-selected indentation as significant
character content?

## Corpus authority

The suite's `doubts.xml` includes an overall instruction that cases using
`xsl:output indent="yes"` need indentation differences handled in the output
comparison stage, except for the whitespace groups. The prior comparator
parsed both outputs as XML but compared every whitespace-only text node
literally, so different permitted indentation widths produced false
mismatches.

## Repair

- Preserve strict whitespace-only text comparison for case identities in the
  dedicated whitespace groups.
- Outside those groups, omit a whitespace-only comparison text node only when
  its parent also contains an element. This targets element indentation while
  preserving text-only whitespace content.
- Continue comparing element and attribute expanded names, attribute values,
  non-whitespace text, comments, processing instructions, child order, and
  ordinary XML line-ending normalization as before.
- Add focused tests proving that two indentation layouts compare equal in the
  relaxed mode, remain unequal in strict mode, and that text-only content is
  not erased.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

Exact XML-semantic matches rise from 1,625 to 1,785. Visible XML mismatches
fall from 217 to 57, and doubt-annotated mismatches fall from 14 to 9.

Initialization remains 2,016 and successful execution remains 1,916. That
unchanged execution envelope confirms this is a comparison-accounting repair,
not newly admitted engine behavior.

## Boundary conclusion

The new lower bound follows an explicit instruction supplied with this corpus.
It does not weaken FastXSLT's engine whitespace semantics, claim that all
whitespace-only result text is insignificant, or establish a general
conformance-comparator rule for other suites. Dedicated whitespace cases and
text-only whitespace remain strict.
