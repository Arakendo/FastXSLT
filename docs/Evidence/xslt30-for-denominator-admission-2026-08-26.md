# XSLT30 `expr/for` Denominator Admission

Date: 2026-08-26

## Inputs

- Suite: W3C XSLT 3.0 test suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/for/_for-test-set.xml`
- Cases: `for-001` through `for-004`
- Dependency retained for every case: `XSLT20+`
- Assertion family: `assert-xml`

## Method

The metadata-driven admission test discovers all four cases from the pinned
test-set document and requires an exact first-party overlay record for each one.
It resolves every stylesheet, referenced environment, principal source,
initial-template declaration, and file-backed or inline assertion.

All four stylesheet files and the three case-specific source admissions are
read with import handles closed, placed under qualified logical identities in a
bounded resource builder, and retained in one sealed snapshot. The file-backed
expected result for `for-001` is also read and checked as a non-empty assertion
input. Engine-classified cases are passed to the current compiler to verify that
their valid-but-unimplemented behavior produces an unsupported category.

That last check exposed and corrected a diagnostic defect: complex valid XPath
syntax such as `sum(for $i in ...)` had fallen through the narrow child-name
parser as invalid input. Syntax characters outside that private grammar now
produce an unsupported classification while structurally invalid ASCII child
names remain invalid.

## Conserved disposition

| Case | Native environment/entry | Assertion | Current disposition | Principal pressure |
| --- | --- | --- | --- | --- |
| `for-001` | `for01` source | file-backed XML | engine unsupported | `xsl:sequence`, `for`, `distinct-values`, comparisons, sequence construction |
| `for-002` | initial template `main`, no principal source | inline XML | harness unsupported | initial-template entry, multiple `for` clauses, arithmetic, `xsl:value-of/@separator` |
| `for-003` | `for03` source | inline XML | engine unsupported | `sum(for ...)` and preservation of focus inside the return expression |
| `for-004` | `for03` source | inline XML | engine unsupported | bound-variable paths, decimal arithmetic, `sum`, and `format-number` |

At this admission checkpoint, the denominator was four selected, zero passed,
three engine-unsupported, one harness-unsupported, zero failed, and zero
metadata failures. `for-002` may also contain unsupported engine semantics, but
the decisive admission-time barrier was that the harness could not invoke its
declared initial template. Native `for-001` subsequently advanced to passed in
[XSLT30 `for-001` Ordered Sequence Execution](xslt30-for-001-ordered-sequence-execution-2026-08-26.md).

## Claim boundary

Admission proves complete inventory, acquisition, metadata retention, bounded
in-memory ownership, and honest present-day classification. It does not prove
`for` expression, sequence, function, numeric, initial-template, or general
XSLT 2.0 support. Future work must advance dispositions in place without
removing a difficult case from the denominator.
