# OASIS XSLT 1.0 Value-Of Tree Expression

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a content-built XSLT 1.0 local variable evaluate the same typed
`xsl:value-of` expression family as an ordinary value-of instruction while
retaining result-tree-fragment behavior?

## Finding

The ordinary typed value evaluator already handled `number($variable)` with
charged XSLT 1.0 variable conversion. The private instruction used for a local
variable whose content is one `xsl:value-of` nevertheless retained only a
`LocationPath`; its compiler therefore rejected every non-path expression
before the shared evaluator could see it.

## Repair

- Retain the already-compiled `ValueExpression` in the private
  `Xslt10ValueOfTreeVariable` instruction instead of a path-only subset.
- Evaluate that expression through the same typed, charged value owner used by
  ordinary `xsl:value-of`, preserving the current focus and lexical variables.
- Materialize the resulting string as the existing invocation-owned temporary
  text tree; do not turn it into an atomic binding.
- Account for the complete typed expression in prepared-program retention.
- Keep the existing path behavior inside the shared evaluator rather than
  creating a second compatibility evaluator.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

A focused nested-template regression passes an atomic parameter into
`number($number)`, constructs a local temporary tree through `xsl:value-of`,
and observes `<out>555</out>` through the ordinary variable value path.

The unchanged `Lotus/namedtemplate_namedtemplate08#1` case becomes an exact
XML-semantic pass. The exact-result lower bound rises from 1,620 to 1,621,
initialization rises from 2,009 to 2,010, and successful execution rises from
1,909 to 1,910.

## Boundary conclusion

This change broadens one existing XSLT 1.0 temporary-text constructor to the
engine's admitted typed value-expression surface. It does not admit general
sequence constructors, change result-tree-fragment identity, add a second
expression evaluator, or alter modern stylesheet semantics.
