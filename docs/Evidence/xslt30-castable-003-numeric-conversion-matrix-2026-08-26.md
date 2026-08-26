# XSLT30 `castable-003` Numeric Conversion Matrix

Date: 2026-08-26

## Native case

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/castable/_castable-test-set.xml`
- Case: `castable-003`
- Dependency: `XSLT20+`
- Environment/source: `castbl01` / `castbl01.xml`
- Stylesheet: `castable-003.xsl`
- Assertion: file-backed `castable-003.out`

The unmodified stylesheet reuses the 12 typed local values introduced by
`castable-002`. It asks 20 positive castability questions across boolean,
integer, decimal, float, and double, followed by three negative questions using
string, duration, and date-time sources. FastXSLT matches the upstream XML
assertion exactly.

## Admitted conversion rules

The private XPath rule table admits these source and target relationships:

```text
boolean                      -> integer | decimal | float | double
integer | decimal            -> integer | decimal | float | double
finite float | finite double -> integer | decimal | float | double
INF | -INF | NaN             -> float | double only
string | untypedAtomic       -> target when its lexical form is accepted
unrelated atomic type        -> incompatible
```

Same-type identity is checked before cross-type conversion. Consequently,
`xs:float('NaN') castable as xs:float` remains true, while a non-finite float or
double is not castable as decimal or integer. Focused tests exercise all 20
native positive edges and the value-sensitive non-finite exclusions.

Each castability question consumes one XPath-operation unit. This checkpoint
does not parse or construct a destination numeric value merely to answer
whether the conversion is permitted.

## Conservation

The complete nine-case denominator remains seven selected and two schema-aware
profile exclusions. Selected execution advances to three passes, one
engine-unsupported case, and three harness-unsupported cases. No case fails or
disappears. Native `castable-004` remains valid-but-unsupported at compilation.

## Claim boundary

This evidence establishes only the castability relationships above. It does
not establish constructed cross-numeric results, public atomic values, numeric
storage, arbitrary precision, implementation range, rounding, overflow,
canonical lexical serialization, arithmetic promotion, or general static type
checking. Duration-family conversion remains the next separate standards slice.
