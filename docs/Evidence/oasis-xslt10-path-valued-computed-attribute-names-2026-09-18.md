# OASIS XSLT 1.0 Path-Valued Computed-Attribute Names

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the exact one-expression path AVT already used for computed-element names
also name `xsl:attribute` results without introducing a general AVT evaluator?

## Result

Yes. Under XSLT 1.0 static context, an `xsl:attribute` name containing exactly
one admitted location path now retains a private typed path, its optional static
namespace URI, and the instruction's static namespace bindings. Execution uses
the charged XPath evaluator and XSLT 1.0 first-node string conversion, then
validates the lexical `QName` with attribute-specific namespace rules.

An empty or malformed result reports `XTDE0850`; reserved `xmlns` names report
`XTDE0855`; invalid prefix/namespace combinations report `XTDE0860`. A static
namespace override is retained on the containing result element at compilation,
so serialization does not invent a binding after semantic construction.

Variable composition, `name(.)`, temporary-tree paths, multiple AVT parts, and
dynamic namespace AVTs remain unsupported.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- Five cases leave `FXST1062`, reducing the frontier from 12 to 7.
- `Lotus/attribset_attribset24#1` and
  `Lotus/attribset_attribset40#1` initialize, execute, and match their
  XML-semantic expected results.
- `Lotus/attribset_attribset15#1` reaches its independent reserved-`xmlns`
  error, while the two Microsoft AVT cases expose a later processing-
  instruction target boundary. Those outcomes receive no pass credit.
- The strict expected-result lower bound rises from 1,567 to 1,569.
- Aggregate lifecycle counts become 1,947 initialized, 1,850 executed
  successfully, 1,188 initialization failures, and 97 execution failures.

## Verification

- Focused runtime tests cover first-node conversion, exact empty-name failure,
  and static namespace-override retention through serialization.
- The complete local OASIS measurement conserves every case identity and
  records the two exact frontier-to-pass transitions.
- The ordinary workspace verification gates pass.
