# QT3 AxisStep and deep-equal conserved denominators -- 2026-09-02

## Result

The pinned QT3 `prod/AxisStep.xml` and `fn/deep-equal.xml` parent sets now have
complete first-party denominator overlays. Every upstream case receives an
effective selection disposition: an existing typed private-ledger record means
`selected/passed`; an unselected case carrying an upstream XQuery-only
specification dependency is `profile-excluded/not-run`; every other sibling
receives the explicit `harness-unsupported/not-run` default. Explicit
case admission has priority over the dependency rule, so `Axes084-5` remains a
selected XPath pass despite its upstream `XQ10+` metadata.

| QT3 test set | Total | Selected and passed | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `prod/AxisStep.xml` | 349 | 189 | 112 | 48 |
| `fn/deep-equal.xml` | 263 | 151 | 67 | 45 |
| **Conserved total** | **612** | **340** | **179** | **93** |

## Mechanical conservation

A typed test-only loader now rejects unknown TOML fields, wrong suite or pinned
revision, empty identities/rationales, dispositions other than
`selected/passed` in the private ledger, duplicate `(set_file, case_name)`
identities, unexpected selected-source references, and denominator defaults
other than `harness-unsupported/not-run`. It also validates nonempty dependency
rules and requires them to produce only `profile-excluded/not-run` outcomes.

The verifier parses each immutable upstream test set, checks the exact 349 and
263 unique case names, proves all 340 selected identities exist in the correct
parent set, reads native per-case dependency metadata, and proves the remaining
179 profile exclusions plus 93 visible defaults. The private ledger is also
checked to contain exactly those 340 cases and no third test-set family.

The AxisStep and deep-equal execution adapters now ask this typed loader for
their selected/pass authority. They no longer establish admission by splitting
TOML text or searching for a case-name substring.

## Boundary

This change does not convert the 93 defaults into engine failures, assert that
their semantics are unsupported by FastXSLT, or classify the other 31,209 QT3
catalog cases. The 179 exclusions describe the current XPath-in-XSLT profile,
not an inability to implement an individual expression; an individual case can
move ahead of the dependency rule only through explicit admission. Case-by-case
promotion still requires an owned evaluator, native metadata validation, and
the native assertion shape.

The selected total includes seven static AxisStep syntax-error cases whose
native `XPST0003` assertions are executed by the location-path parser.
[Evidence](qt3-axis-static-syntax-errors-2026-09-02.md)
