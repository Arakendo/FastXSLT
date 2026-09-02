# QT3 AxisStep and deep-equal conserved denominators -- 2026-09-02

## Result

The pinned QT3 `prod/AxisStep.xml` and `fn/deep-equal.xml` parent sets now have
complete first-party denominator overlays. Every upstream case receives an
effective selection disposition: an existing typed private-ledger record means
`selected/passed`; every other sibling receives the explicit
`harness-unsupported/not-run` default.

| QT3 test set | Total | Selected and passed | Visible not run |
| --- | ---: | ---: | ---: |
| `prod/AxisStep.xml` | 349 | 182 | 167 |
| `fn/deep-equal.xml` | 263 | 151 | 112 |
| **Conserved total** | **612** | **333** | **279** |

## Mechanical conservation

A typed test-only loader now rejects unknown TOML fields, wrong suite or pinned
revision, empty identities/rationales, dispositions other than
`selected/passed` in the private ledger, duplicate `(set_file, case_name)`
identities, unexpected selected-source references, and denominator defaults
other than `harness-unsupported/not-run`.

The verifier parses each immutable upstream test set, checks the exact 349 and
263 unique case names, proves all 333 selected identities exist in the correct
parent set, and accounts for the remaining 279 cases through the visible
default. The private ledger is also checked to contain exactly those 333 cases
and no third test-set family.

## Boundary

This change does not convert the 279 defaults into engine failures, assert that
their semantics are unsupported by FastXSLT, or classify the other 31,209 QT3
catalog cases. It establishes honest selection accounting for the two parent
sets already under active execution. Case-by-case promotion still requires an
owned evaluator, native metadata validation, and the native assertion shape.
