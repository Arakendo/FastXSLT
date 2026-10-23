# XSLT30 `castable-004` Duration Conversion Matrix

Date: 2026-08-26

## Native case

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/castable/_castable-test-set.xml`
- Case: `castable-004`
- Dependency: `XSLT20+`
- Environment/source: `castbl01` / `castbl01.xml`
- Stylesheet: `castable-004.xsl`
- Assertion: inline `assert-xml`

The unmodified stylesheet reuses the typed local values introduced by
`castable-002`. It asks four positive questions among `xs:duration`,
`xs:dayTimeDuration`, and `xs:yearMonthDuration`, followed by four negative
questions using date, time, boolean, and integer sources. FastXSLT produces the
eight expected Boolean result elements.

## Admitted conversion rules

The private XPath rule table treats all three duration-family types as mutually
castable. It does not extend that relationship to other temporal or numeric
types:

```text
duration-family -> duration-family = true
date             -> yearMonthDuration = false
time             -> dayTimeDuration = false
boolean          -> yearMonthDuration = false
integer          -> dayTimeDuration = false
```

String and untyped values retain the earlier lexical, value-dependent rule.
Each castability question consumes one XPath-operation unit. No converted
duration value is constructed merely to answer castability.

## Inline assertion adapter

The first three native cases use file-backed XML assertions containing XML
declarations; `castable-004` supplies inline XML without one. The focused
adapter verifies that both strings are well-formed XML, removes only an optional
leading XML declaration, and then compares the remaining serialization exactly.
It does not claim general `assert-xml` equivalence for prefix changes, attribute
ordering, whitespace policy, comments, processing instructions, or other test
suite options.

## Conservation

The complete nine-case denominator remains seven selected and two schema-aware
profile exclusions. Selected execution advances to four passes, no
engine-unsupported cases, and three harness-unsupported cases. No case fails or
disappears.

## Claim boundary

This evidence establishes castability only. It does not establish constructed
duration conversion results, public atomic values, duration storage or
normalization, arithmetic, comparison, canonical lexical serialization, or a
general W3C XML assertion evaluator. Cases `castable-007` through `009` retain
their compound-assertion harness dispositions and are not counted as engine
passes.
