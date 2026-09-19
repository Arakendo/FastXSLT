# OASIS XSLT 1.0 Variable `contains()`

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing XSLT 1.0 variable conversion model admit
`contains($nodeSet, $string)` without adding a general expression evaluator?

## Finding

FastXSLT already owned both required conversions: first-node string value for a
source node-set variable and lexical string value for an atomic variable. The
compiler nevertheless sent the first operand through the path-string-function
parser, which correctly rejected `$node` as not being a location path.

## Repair

- Add one typed XSLT 1.0 plan for `contains()` with exactly two unqualified
  variable references.
- Select it before the existing path/literal string-function family.
- Resolve both operands through the shared charged XSLT 1.0 variable string
  conversion.
- Charge the function operation and reuse ordinary boolean result construction.
- Include both owned variable names in compiled-capacity accounting.
- Retain all other argument shapes at their existing explicit boundaries.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

The focused mixed-kind case converts a local source-node variable containing
`AB-CY-DE` and a global atomic variable containing `CY`, producing `true`.

The unchanged `Lotus/string_string56#1` case becomes an exact XML-semantic
pass. The exact-result lower bound rises from 1,619 to 1,620, initialization
rises from 2,008 to 2,009, and successful execution rises from 1,908 to 1,909.
All execution-failure and comparison-frontier totals remain unchanged.

## Boundary conclusion

This is a compatibility plan selected only for XSLT 1.0 static context. It does
not change modern function conversion, admit literal/path mixtures, generalize
function calls, or introduce a second variable representation.

