# OASIS XSLT 1.0 Formatted-Number Conversion

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XPath 1.0 `number()` consume the string produced by the existing typed
`format-number()` implementation without introducing a general dynamic
expression tree or a second formatting path?

## Repair

- Compile `number(format-number(...))` only in XSLT 1.0 compatibility mode and
  retain the existing typed `FormatNumberExpression` as its operand.
- Reuse the same default/named decimal-format binding pass and runtime variable
  conversion as ordinary `format-number()`.
- Convert the formatted string with the existing XPath 1.0 number lexical rules
  and emit the canonical XPath 1.0 number lexical.
- Account for the retained formatting expression in compiled-state capacity.

## Result

A focused runtime regression proves that `format-number(-1234.5, '###0.00 ')`
produces `-1234.50 ` while wrapping the same expression in `number()` produces
`-1234.5`.

The unchanged Microsoft `XSLTFunctions__testWithNumber#1` case now initializes
and reaches serialization. Its requested ISO-8859-1 output remains explicitly
unsupported by the transform set's UTF-8 string-result lane, so it is not
credited as an execution or comparison pass. The complete 3,173-case sweep
moves to 2,026 initialized cases while successful executions remain 1,925,
exact XML-semantic matches remain 1,794, and mismatches remain 57.

## Boundary conclusion

This is one typed XSLT 1.0 conversion composition. It does not admit arbitrary
nested dynamic expressions, relax the existing `format-number()` operand and
picture bounds, or make a UTF-8 string result pretend to satisfy a legacy byte
encoding request. The latter requires the existing bounded byte-serialization
boundary to be integrated deliberately with corpus execution.
