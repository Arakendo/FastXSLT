# OASIS XSLT 1.0 Path-Valued Computed-Element Names

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the remaining dynamic `xsl:element` frontier admit a bounded useful family
without introducing a general string-expression evaluator or a second result
construction path?

## Result

Yes. A name AVT containing exactly one admitted location path now compiles into
a typed path-name element instruction. Execution evaluates that path through
the ordinary charged XPath owner, applies XSLT 1.0 first-node string conversion,
validates the resulting lexical `QName`, resolves any prefix against the
instruction's retained static namespace context, and then uses the existing
semantic result-element constructor.

The slice includes forms such as `{@style}`, `{.//first-name}`, `{//last-name}`,
and `{p2:BBB/@id}` when their paths are in the admitted grammar. Empty or
malformed runtime names report `XTDE0820`; unbound prefixes report `XTDE0830`;
and incompatible namespace overrides remain structured errors. Variable paths,
mixed text/expression names such as `e{position()}`, function predicates, and
dynamic namespace AVTs remain unsupported.

The runtime resolver is a private one-way module. It owns only path evaluation,
lexical-name validation, and namespace resolution; result construction,
stylesheet compilation, host policy, and public representation remain outside
it.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- Eleven cases leave `FXST1047`, reducing that frontier from 13 to 2.
- Seven encounter visible later initialization frontiers.
- Four initialize; three execute successfully and one reports the expected
  runtime `XTDE0820` for an empty name.
- `Lotus/lre_lre08#1` becomes an exact XML-semantic match, raising the strict
  expected-result lower bound from 1,565 to 1,566.
- Two successful cases remain visible comparison mismatches.
- Aggregate lifecycle counts become 1,944 initialized, 1,847 executed
  successfully, 1,191 initialization failures, and 97 execution failures.

This is a bounded shared XPath/result-construction primitive, not general
dynamic `QName` or AVT support.

## Verification

- Focused compiler coverage proves `{@name}` lowers to the typed path-name
  instruction while malformed static names retain `XTDE0820` classification.
- Focused runtime coverage proves unprefixed and statically bound prefixed name
  construction plus runtime malformed-name rejection.
- The complete local OASIS measurement conserves every case identity and
  records all eleven frontier transitions.
- The ordinary workspace verification gates pass.
