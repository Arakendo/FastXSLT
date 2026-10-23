# QT3 deep-equal String-Derived NCName -- 2026-09-02

## Result

Unchanged QT3 case `K2-SeqDeepEqualFunc-35` now executes through the private
`deep-equal` atomic-value path. The case compares the string literal `"a"`
with `xs:NCName("a")` and expects `true`.

The atomic parser admits the ASCII NCName lexical subset, validates the lexical
form before construction, and compares the resulting string-derived value by
value with `xs:string`. A focused control rejects a lexical form beginning with
a digit. This does not admit the other XML Schema string-derived constructors,
Unicode NCName coverage, general casting, or schema-aware typed nodes.

## Accounting

The `fn/deep-equal.xml` denominator advances from 151 to 152 passes, retains 67
XQuery-profile exclusions, and reduces visible default not-run cases from 45
to 44. Across the two active QT3 denominators, the conserved subtotal is now
341 passes, 179 profile exclusions, and 92 visible default not-run cases out of
612.

