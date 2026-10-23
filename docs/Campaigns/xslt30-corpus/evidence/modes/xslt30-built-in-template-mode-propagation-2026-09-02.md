# XSLT30 Built-In Template Mode Propagation

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Native test set
  `tests/misc/built-in-templates/_built-in-templates-test-set.xml`.
- Unchanged cases `built-in-templates-0101` and
  `built-in-templates-0102`, their external source, stylesheets, and native
  XML assertions.

## Method

The adapter inventories all six native cases before selection. A typed overlay
gives the denominator a visible `harness-unsupported/not-run` default and
selects only the two mode-propagation cases. Each selected case must also have
an exact selected/pass entry in the private case ledger.

For each selected case, the adapter reads the unchanged external source and
stylesheet into a bounded sealed snapshot, compiles once, and executes one
principal-source request through the normal transform-set path. The result is
compared with the native `assert-xml` expectation after XML line-ending
normalization; this avoids treating the external expectation's CRLF bytes as
semantic text differences.

## Results

- Complete conserved denominator: 6 cases.
- Selected and passed: 2 (`built-in-templates-0101` and `0102`).
- Visible default not run: 4.
- Engine unsupported: 0.
- Profile excluded: 0.
- Focused adapter tests: 2 passed.

The unchanged cases show that `mode="#current"` retains the active unnamed
mode through recursive built-in document and element rules, while
`mode="#default"` explicitly selects that same unnamed mode. Descendant
processing-instruction and element templates are reached according to the
selected mode; templates declared only in unrelated modes do not leak into
dispatch.

Adding this denominator changes conserved XSLT30 accounting to 567 cases: 409
passed comparisons, 3 engine-unsupported cases, 54 profile exclusions, and 101
visible default not-run cases.

## Limitations

The four default cases require parameter propagation with typed or
sequence-constructor values, or schema annotation behavior. This evidence does
not admit those semantics and does not claim schema-aware execution. It also
does not turn the byte serializer into the authority for `assert-xml`; the
comparison remains scoped to the XML assertion family exercised here.
