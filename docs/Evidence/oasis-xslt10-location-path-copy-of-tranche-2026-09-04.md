# OASIS XSLT 1.0 Location-Path Copy-Of Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Archive SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Input baseline | 386 definite unchanged XML passes |
| Result | 395 definite unchanged XML passes |
| Disposition | Shared node-path `xsl:copy-of` slice implemented; general sequence copying remains open |

## Outcome

`xsl:copy-of` can now compile ordinary node-selecting location paths through the
existing typed path parser and execute them through the charged path evaluator.
Each selected document, element, or text node is copied using the existing deep
source-copy operation, preserving selection order, element names, namespace
bindings, attributes, descendants, text, work budgets, and cancellation.

The complete local OASIS sweep moved as follows:

| Observation | Prior tranche | This tranche | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 540 | 635 | +95 |
| Engine initialization failed | 2,595 | 2,500 | -95 |
| Execution succeeded | 436 | 450 | +14 |
| Execution failed | 104 | 185 | +81 |
| Definite unchanged XML passes | 386 | 395 | +9 |
| XML comparison mismatches | 25 | 29 | +4 |
| Comparator unsupported | 15 | 15 | 0 |

The strict standard-operation lower bound is now **395 / 2,742 = 14.41%**;
the complete-catalog lower bound is **395 / 3,173 = 12.45%**. The large
initialization movement and smaller pass movement are useful evidence: 81 cases
now expose a later runtime or serialization boundary. They are not counted as
progress merely because compilation reached farther.

Unchanged `Microsoft/Copying__84388#1` now executes its
`/bookstore/book/author/first-name` selection and reaches a successful XML
comparison. A focused production regression separately selects two nonadjacent
`/doc/selected` elements, deep-copies their attributes and descendants, and
asserts document-order output.

## Architectural conservation

- The compiler owns XPath parsing and lowers one typed `LocationPath`; the
  runtime does not parse expression text.
- Evaluation reuses the existing bounded XPath path evaluator and deep-copy
  implementation rather than creating an XSLT 1.0 evaluator.
- Selected-node normalization, namespace retention, result-node charging,
  cancellation, and serialization remain shared modern-engine behavior.
- Compiled-state retention accounting includes the owned path capacity, and
  semantic inspection continues to report the operation as `CopyOf` without
  exposing its private instruction shape.

## Deliberately open forms

This is a node-path slice, not general XPath sequence-copy semantics. The
remaining 29 compile-time `FXXP1003` cases use expression forms outside the
admitted path grammar. Paths selecting source attributes, comments, or
processing instructions currently reach an explicit `FXRT1002` unsupported
node-kind boundary rather than producing a plausible partial result. Atomic
values, variables, unions, computed sequences, namespace-copy controls, and
validation/type behavior remain open.

## Next work

Classify the 81 newly exposed execution failures before widening `copy-of`
again. The result confirms that first-failure removal alone is not a useful
completion metric. The next primitive should be selected by reusable semantic
leverage across the OASIS and modern suites, while later `copy-of` work should
separate node-kind copying from general typed-sequence copying explicitly.
