# XSLT30 `for-004` Exact Decimal and Complete `for` Denominator

Date: 2026-08-26

## Native case

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/for/_for-test-set.xml`
- Case: `for-004`
- Dependency: `XSLT20+`
- Environment/source: `for03` / `for03.xml`
- Stylesheet: `for-004.xsl`
- Assertion: inline `<out>36.02</out>`

The native expression is:

```xpath
format-number(
  sum(for $i in order-item return $i/@price * $i/@qty),
  '0.00')
```

Unlike `for-003`, both attribute steps are rooted at the bound `$i` item. The
outer focus remains the matched `order`; variable binding supplies a distinct
navigation origin without silently changing that focus.

## Exact result

The private evaluator represents each finite decimal as a checked integer
mantissa plus scale. It does not convert the native values through binary
floating point. The five products aggregate exactly:

```text
11.32 * 1 + 2.34 * 3 + 1.00 * 5 + 2.56 * 3 + 5.00 * 1 = 36.02
```

The result then passes through only the `format-number` picture `'0.00'`. A
value requiring discarded nonzero decimal places is refused as unsupported;
the implementation does not invent unverified rounding behavior. Checked
mantissa, scale, multiplication, alignment, and addition overflow also remain
unsupported rather than wrapping.

## Bounded work

Attribute navigation consumes the XPath node-visit domain and attribute string
conversion consumes the XDM string-value-node domain. Each product and
aggregation is charged independently, followed by one formatting operation.
The five native tuples therefore consume 11 XPath-operation units.

## Conservation and claim boundary

All four selected cases in the complete pinned XSLT30 `expr/for` test set now
pass: four passed, zero engine-unsupported, zero harness-unsupported, and zero
failed. This is a conserved local denominator, not a general XSLT/XPath
conformance claim.

The evidence does not establish general decimal types, numeric promotion,
exponents, rounding modes, arbitrary formatting pictures, general function
calls, or generalized FLWOR expressions. Those remain unsupported until a
native case and a deliberate semantic slice require them.
