# OASIS XSLT 1.0 Variable `sum()`

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `sum($variable)` reuse the existing typed source-node variable and charged
path-sum semantics without adding a general expression evaluator or converting
temporary trees into node-sets?

## Finding

FastXSLT already retained local path selections as source node IDs and already
implemented XSLT 1.0 numeric conversion and accumulation for `sum(path)`.
Compilation nevertheless required the `sum()` operand itself to be a location
path, so a variable reference could not reach either existing owner.

## Repair

- Add one typed XSLT 1.0 value plan for `sum()` with exactly one unqualified
  variable reference.
- Require that variable to resolve to a source-node sequence; atomic values and
  temporary trees retain an explicit `XPTY0004` boundary.
- Share the charged node string-value conversion, XPath numeric conversion,
  accumulation, and lexical result formatting with the existing path plan.
- Retain and account for only the variable name in the compiled plan.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

A focused regression binds the three source nodes `1`, `2`, and `3` to a local
variable and observes `<out>6</out>` from `sum($values)`.

The unchanged, doubt-annotated `Lotus/math_math84#1` case becomes an exact
XML-semantic pass. The exact-result lower bound rises from 1,621 to 1,622,
initialization rises from 2,010 to 2,011, and successful execution rises from
1,910 to 1,911. The suite's doubt annotation remains visible; this result is
compatibility evidence and does not resolve the upstream case's historical
status.

## Boundary conclusion

This is a typed XSLT 1.0 operand specialization over the engine's existing
source-node representation. It does not permit result-tree-fragment node-set
conversion, widen modern function conversion, introduce retained indexes, or
add another numeric evaluator.
