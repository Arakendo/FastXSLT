# OASIS XSLT 1.0 `contains()` over Typed `concat()` Operands

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can XSLT 1.0 `contains()` compose two already supported dynamic `concat()`
operands without introducing a general expression evaluator?

## Implemented boundary

The XSLT 1.0 value compiler now recognizes the exact two-argument shape
`contains(concat(...), concat(...))`. Both operands compile to the existing
typed concat representation, including its bounded argument count, source
context, variable lookup, path evaluation, and work charging. Runtime evaluates
the two strings and charges the containment test before emitting its boolean
lexical result.

The change does not add general nested function composition, change modern
XPath behavior, or create another string evaluator. An equivalent version 3.0
expression remains explicitly unsupported by the current modern slice.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,238 to 2,239;
- successful executions rise from 2,178 to 2,179;
- exact XML comparisons rise from 2,029 to 2,030 / 3,173 (63.98%);
- unchanged `Lotus/string_string57#1` now produces the expected
  `<out>true</out>` result; and
- execution failures remain 60, comparison mismatches remain 66,
  expected-error unexpected successes remain five, and no execution panic
  occurs.

## Verification

A focused compiler test proves that only the XSLT 1.0 profile selects the typed
composition. A focused runtime test evaluates a context-derived first concat
operand and a literal second concat operand. The unchanged corpus case then
exercises the same path through compilation, execution, serialization, and
XML-semantic comparison.

The archive remains local and unmodified. This is compatibility evidence, not
a broad conformance claim.
