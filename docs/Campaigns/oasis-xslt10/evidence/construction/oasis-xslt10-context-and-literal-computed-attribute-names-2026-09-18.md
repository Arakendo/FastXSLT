# OASIS XSLT 1.0 Context and Literal Computed-Attribute Names

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the bounded dynamic computed-attribute name plan also represent exact
`name(.)` and one-expression string-literal AVTs without admitting a general
AVT evaluator?

## Result

Yes. Under XSLT 1.0 static context, `name()` / `name(.)` now reads the lexical
name of the source focus through charged node access. A one-expression XPath
string literal is folded at compilation. Both values then use the same
attribute-specific runtime `QName` and namespace validation as path-valued
computed names.

Variable composition, temporary-tree paths, multiple general AVT parts, and
dynamic namespace AVTs remained outside this tranche. The separately measured
variable-only follow-up reuses the existing runtime variable frame without
changing the result recorded here.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- Two more cases leave `FXST1062`, reducing the frontier from 7 to 5.
- `Microsoft/Attributes_Attribute_InvalidNamespacePrefix#1` now initializes
  and reports the required runtime `XTDE0855` for an `xmlns`-prefixed computed
  name.
- `Lotus/copy_copy07#1` now initializes and reaches the independent existing
  `XTDE0410` result-attribute attachment boundary.
- Neither case receives pass credit. The strict expected-result lower bound
  remains 1,569.
- Aggregate lifecycle counts become 1,949 initialized, 1,850 executed
  successfully, 1,186 initialization failures, and 99 execution failures.
  One expected error moves from initialization to execution, preserving its
  identity and expected-error disposition.

## Verification

- Focused runtime tests cover source-context lexical-name construction and the
  reserved-`xmlns` failure for a folded literal AVT.
- The complete local OASIS measurement conserves every case identity and keeps
  the two later failures visible.
- The ordinary workspace verification gates pass.
