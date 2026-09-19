# OASIS XSLT 1.0 Static Computed-Element QName Errors

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can statically malformed `xsl:element` names be distinguished from valid
dynamic name AVTs before general dynamic-name execution is implemented?

## Result

Yes. After excluding name values containing AVT delimiters, compilation now
validates the remaining lexical value as a QName. Empty names, whitespace,
missing prefix/local parts, multiple colons, slash-only names, and other
statically malformed values report `XTDE0820 / invalid` with the instruction's
source location.

Names containing AVT delimiters remain `FXST1047 / unsupported` unless they
belong to the already admitted exact context-name forms. This does not evaluate
an arbitrary name AVT or infer a dynamic value from its lexical spelling.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- Ten unchanged cases move from `unsupported/FXST1047` to
  `invalid/XTDE0820`, covering empty, whitespace-bearing, missing-part,
  multiple-colon, and slash-only lexical names.
- The `FXST1047` frontier falls from 23 to 13 cases. Every survivor contains a
  genuinely dynamic name expression.
- Aggregate lifecycle and comparison counts are unchanged: 1,940 cases
  initialize, 1,844 execute successfully, and 1,565 compare exactly.

This is standards diagnostic refinement, not a pass-count increase.

## Verification

- A focused compiler test covers five malformed static QName classes and proves
  an attribute-valued name AVT remains unsupported.
- The complete local OASIS measurement conserves every case identity and
  reports exactly ten `XTDE0820` transitions.
- The ordinary workspace verification gates pass.
