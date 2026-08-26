# XSLT30 Case-Metadata Inventory

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Command | `./scripts/inventory-xslt30-case-metadata.ps1` |
| Decision pressure | AR-0001 staged preview denominator |
| Claim | Structural inventory only; no support or conformance claim |

## Question

Does the pinned XSLT30 suite contain enough complete case metadata to drive a
testable standards-based FastXSLT preview before representative consumer
stylesheets arrive, and what harness capabilities dominate that corpus?

## Method

The inventory verifies the pinned submodule revision, traverses all 234 root
catalog test-set references, rejects paths escaping the suite root, and reads
XML with DTD processing prohibited and no resolver. It records test cases,
stylesheet references, dependency element kinds and explicit `spec` values,
environment binding shapes, top-level and nested assertion families, and the
number of distinct combined metadata shapes.

The default JSON remains aggregate and reviewable. The script can include all
564 metadata shapes with `-IncludeMetadataShapes` when selection work needs the
full combinations.

## Results

| Observation | Count |
| --- | ---: |
| Test sets | 234 |
| Test cases | 14,600 |
| Stylesheet references from cases | 9,663 |
| Distinct case-referenced stylesheet files | 7,646 |
| Distinct dependency/assertion/environment/stylesheet shapes | 564 |
| Referenced environments | 10,798 |
| Inline environments | 2,161 |
| Cases without an environment binding | 1,641 |

The repository contains 8,787 `.xsl`/`.xslt` files in total. The smaller 7,646
count is deliberately limited to distinct files reached through case-local
`stylesheet file` references; other suite files may be support modules,
alternate inputs, environment-owned resources, or unreferenced material and do
not become preview cases merely because they exist.

Explicit standards dependencies occur as follows:

| `spec` value | Occurrences |
| --- | ---: |
| `XSLT10` | 1 |
| `XSLT10 XSLT20` | 21 |
| `XSLT10+` | 2,016 |
| `XSLT20` | 111 |
| `XSLT20+` | 3,359 |
| `XSLT30+` | 1,371 |

The remaining cases have no case-local `spec` dependency. Absence remains a
classification fact; the inventory does not invent a default edition. There
are 22 dependency element kinds. Besides `spec`, the largest general kind is
`feature` with 1,817 occurrences; other kinds cover implementation-defined or
environmental choices such as numbering, normalization, output encoding,
assertions, packages, calendars, available documents, and extension functions.

The largest top-level result assertion families are:

| Assertion | Occurrences |
| --- | ---: |
| `assert` | 5,479 |
| `assert-xml` | 4,927 |
| `error` | 2,364 |
| `all-of` | 1,045 |
| `any-of` | 332 |
| `assert-string-value` | 164 |
| `serialization-matches` | 124 |
| `assert-serialization` | 84 |

Nested assertions add message, result-document, serialization-error, warning,
type, equality, count, and deep-equality requirements. A preview harness cannot
honestly reduce every result to byte equality.

## Findings

- The suite is sufficient to drive standards-based preview work now. Consumer
  artifacts are not required to discover complete stylesheets, sources,
  dependencies, environments, assertions, and expected errors.
- Metadata classification must precede semantic execution. The 564 shapes show
  why selecting files by apparent stylesheet simplicity would hide material
  dependencies and comparison requirements.
- `assert-xml` is the strongest first positive-result comparison family because
  it is large, already appears in the executed `template-006` path, and does
  not confuse equivalent XML serialization with exact bytes.
- Expected-error handling is also a major denominator and must remain distinct
  from harness or engine operation failure.
- The generic `assert` family is larger than `assert-xml`, but it requires XPath
  evaluation of suite assertions and therefore should not be mistaken for an
  inexpensive first comparator.
- Consumer transforms still determine product priority, useful compatibility,
  host lifecycle, and representative performance. They no longer block a
  testable standards-driven preview.

## Limitations

This inventory does not parse stylesheet semantics, resolve environment
resources, classify inherited or implicit suite meaning beyond the recorded
elements, execute a case, or prove that a metadata-simple case fits the current
engine. The next selection must combine complete metadata with actual FastXSLT
compile/execute outcomes and retain every non-selected disposition under
ADR-0006.
