# OASIS XSLT 1.0 Concatenated `translate()` Maps

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XSLT 1.0 `translate()` accept an existing bounded `concat()` expression as
either mapping operand while retaining a typed, compile-time-selected plan?

## Finding

FastXSLT already compiled a source path plus two literal translation maps and
already evaluated bounded `concat()` parts containing literals, variables,
paths, and path sums. The two expression families could not yet compose, so
stylesheets that assembled a quote/apostrophe map from variables were rejected
before execution.

## Repair

- Add a typed composed path-translation plan whose search and replacement
  operands are each either a literal or an existing compiled concat plan.
- Require at least one concat operand; retain the smaller literal-only plan for
  the ordinary fast path.
- Resolve the source path with the stylesheet's static namespace context.
- Reuse charged concat evaluation, first-node path string conversion,
  Unicode-codepoint translation, and bounded text-result construction.
- Include the path and nested operand ownership in compiled-capacity
  accounting.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

A focused regression exercises concatenated variable maps in both directions:
as the search map it translates `abc` to `XYZ`, and as the replacement map it
translates `XYZ` back to `abc`.

The unchanged `Lotus/string_string138#1` and
`Lotus/string_string139#1` cases become exact XML-semantic passes. The
exact-result lower bound rises from 1,623 to 1,625, initialization rises from
2,012 to 2,014, and successful execution rises from 1,912 to 1,914.

## Boundary conclusion

This composes two existing typed XSLT 1.0 plans. It does not admit arbitrary
function-valued operands, dynamic function dispatch, non-path input values, or
changes to modern `translate()` semantics.
