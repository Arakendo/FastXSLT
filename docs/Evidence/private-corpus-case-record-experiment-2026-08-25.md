# Private Corpus Case-Record Experiment

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| QT3 revision | `83993587711dbd5c18ed846385ec37d079d6e492` |
| XSLT30 revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Implementation | `crates/fastxslt/src/verification_ledger_experiment.rs` |
| Claim | Private metadata/classification evidence; no conformance claim |

## Inputs

The experiment reads two unmodified upstream test-set documents:

| Suite | Native case | Environment | Assertion shape | Private disposition |
| --- | --- | --- | --- | --- |
| QT3 | `prod/AxisStep.xml#Axes001-1` | `TreeTrunc` | `assert-eq` | selected; engine unsupported |
| XSLT30 | `tests/attr/avt/_avt-test-set.xml#avt-0701` | `avt-07` | `all-of` containing `assert-xml` and `assert-message/assert-eq` | selected; harness unsupported |

The XSLT30 case also retains stylesheet `avt-0701.xsl` and dependency
`XSLT20+`. The first-party overlays retain selection, execution disposition,
and rationale without changing either submodule.

## Method

Test-only suite adapters parse each test-set into the private owned XDM tree and
produce different suite-native observation records. They preserve native case,
test-set, environment, assertion, and immutable suite revision identity. Only a
small reporting projection is shared: identity, selection disposition, and
execution disposition. QT3 expression metadata and XSLT30 stylesheet,
dependency, and nested assertion paths remain suite-specific.

Focused tests verify the exact upstream metadata and two classifications. A
synthetic future QT3 assertion name and a synthetic future XSLT30 dependency
both produce `harness unsupported`. They cannot disappear as exclusions or be
interpreted as passes.

## Result

Two materially different assertion/environment families can feed one minimal
case-record projection without imposing a universal normalized corpus schema.
The result also separates two often-confused outcomes:

- the harness understands QT3 `assert-eq`, but the current private XPath engine
  does not implement the selected expression; and
- the XSLT30 compound message assertion requires comparison behavior the
  private harness does not yet own.

The focused tests pass, and the workspace contains 44 tests in total, including
one ignored manual filesystem-release probe.

## Limitations

- The experiment observes metadata; it does not execute either new case.
- It does not establish AR-0001 applicability or a standards denominator.
- It does not yet map the referenced environments into sealed snapshots.
- It does not prove filtered, sharded, interrupted, retried, or merged ledger
  conservation.
- Its private Rust records and TOML fields are not stable schemas or public
  APIs.
