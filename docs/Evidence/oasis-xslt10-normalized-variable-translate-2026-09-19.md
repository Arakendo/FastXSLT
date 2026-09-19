# OASIS XSLT 1.0 Normalized Variable Translation

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the bounded XSLT 1.0 value compiler compose
`translate(normalize-space($sourceNodes), literal, literal)` from existing
normalization and translation owners without introducing a general expression
interpreter?

## Finding

FastXSLT already streamed XML-whitespace normalization over the first node of a
source-node variable and already implemented Unicode-codepoint `translate()`
for a path plus two literal mapping strings. The compiler admitted both as
standalone shapes but did not retain their nested composition.

## Repair

- Add one typed XSLT 1.0 plan for a normalized source-node variable followed by
  two literal `translate()` mapping operands.
- Resolve the variable through the existing lexical source-node frame and
  preserve first-node-in-document-order conversion.
- Reuse streaming, charged XML-whitespace normalization and the established
  codepoint translation implementation.
- Charge the translation operation and use ordinary bounded text-result
  construction.
- Retain all three owned strings in compiled-capacity accounting.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

A focused regression normalizes `  first&#x9; second  ` through a source-node
variable and translates the remaining space to `_`, producing
`<out>first_second</out>`.

The unchanged `Lotus/string_string121#1` case becomes an exact XML-semantic
pass. The exact-result lower bound rises from 1,622 to 1,623, initialization
rises from 2,011 to 2,012, and successful execution rises from 1,911 to 1,912.

## Boundary conclusion

This is a compile-time-selected XSLT 1.0 composition over two existing safe
semantic owners. It does not admit dynamic mapping operands, arbitrary nested
function calls, atomic or temporary-tree variable normalization, or any change
to modern function conversion.
