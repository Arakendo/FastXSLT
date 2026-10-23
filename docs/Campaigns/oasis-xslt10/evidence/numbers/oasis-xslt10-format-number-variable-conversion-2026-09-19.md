# OASIS XSLT 1.0 `format-number()` Variable Conversion

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Why did `format-number($number, $picture)` report an unbound variable when the
same local variables were immediately visible to neighboring `xsl:value-of`
instructions?

## Finding

The runtime formatter copied only invocation-local atomic bindings into its
private operand map. It separately converted temporary result trees, but it did
not admit global atomics, atomic sequences, or source-node-set variables. The
ordinary XSLT 1.0 value runtime already owned the required first-item/string-
value conversion, global fallback, shadowing, work charging, and cancellation.

The Microsoft fixtures bind both the number and picture as source node sets:
the number selects an attribute and the picture selects the current element.
Those are valid XSLT 1.0 operands and were incorrectly diagnosed as unbound.

## Repair

- Resolve every referenced formatter variable missing from the local atomic
  map through the existing charged XSLT 1.0 variable string conversion.
- Preserve native atomic bindings without conversion or a second lookup path.
- Preserve the bounded formatter, decimal-format selection, error mapping,
  budgets, cancellation, lexical scope, and result construction.
- Add a focused local source-node-variable case for both number and picture.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

The focused case formats source attributes `1234.5` and `#,##0.00` as
`1,234.50`.

Seven unchanged Microsoft cases leave the misleading `FXRT0002 / unbound
variable` frontier. Six reach the honest bounded dynamic-picture formatter
frontier `FXRT1007`; the expected-error currency case completes expression
execution and reaches its independent ISO-8859-1 string-serialization boundary.
The `FXRT0002` frontier falls from 20 to 13 and `FXRT1007` rises from 12 to 18.

The exact-result lower bound remains 1,618, with 2,008 initialized cases, 1,903
successful executions, and 214 visible comparison mismatches. The broader
formatter frontier includes arbitrary-precision decimal input, invalid dynamic
pictures, named decimal-format behavior, and other work that this repair does
not silently admit.

## Boundary conclusion

This removes a consumer-specific variable-kind hole by reusing the established
XSLT 1.0 conversion owner. It does not add a general expression evaluator,
expand the formatter grammar, select arbitrary-precision storage, or weaken an
unsupported boundary to obtain a corpus pass.

