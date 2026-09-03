# QT3 Effective-Boolean-Value Error Tranche

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Nine unchanged cases from the already conserved `fn/not.xml` and
  `fn/boolean.xml` denominators.

## Method

The private atomic EBV operation now distinguishes three outcomes: a defined
boolean value, a parsed value or sequence for which XPath defines no effective
boolean value, and syntax outside the admitted grammar. The boolean parser maps
only the second outcome to the native `FORG0006` dynamic error. It continues to
map invalid function arity to `XPST0017` and leaves unparsed syntax unsupported.

The admitted error cases cover two-boolean, two-integer, and two-string
sequences plus singleton `xs:dateTime`, `xs:QName`, `xs:hexBinary`, and
`xs:base64Binary` values. The QName addition accepts only an unprefixed quoted
lexical QName in this source-free context; it does not invent namespace
bindings. Parsing an outer parenthesized sequence before recursively
interpreting its items also prevents `true(), false()` from being mistaken for
a malformed zero-arity function call.

## Result

| Test set | Added passes | Current passes | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `fn/not.xml` | 1 | 74 | 3 | 6 |
| `fn/boolean.xml` | 8 | 122 | 5 | 16 |
| **Combined** | **9** | **196** | **8** | **22** |

The overall audited QT3 denominator remains 1,000 cases and now contains 747
passes, 191 profile exclusions, and 62 visible default not-run cases.

## Boundary

This tranche does not claim mixed node/atomic sequence EBV, function/map/array
items, or general runtime error delivery. Those cases remain visible defaults.
