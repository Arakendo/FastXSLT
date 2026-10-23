# XSLT30 Explicit Conflict-Recovery Variants

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Cases: `conflict-resolution-0102a`, `0104a`, `0108a`, `0110a`,
  `0401a`, and `1202a`

## Method and result

Each case reuses a stylesheet already executed by its XSLT 3.0+ `c` variant,
but the cases have different native applicability metadata. A focused test
reads the pinned test-set document and requires `on-multiple-match=recover` for
all six cases. It also conserves `spec=XSLT10 XSLT20` for five cases and
`spec=XSLT20` for `0401a` before compiling and executing the case.

| Cases | Conflict shape | Result |
| --- | --- | --- |
| `0102a`, `0104a` | equal-priority wildcard/node tests | passed |
| `0108a`, `0110a` | equal-priority non-simple patterns | passed |
| `0401a` | equal explicit priority and namespace wildcard | passed |
| `1202a` | equal-rank entry plus `xsl:next-match` continuation | passed |

Every serialized result is semantically equal to its pinned `assert-xml`
outcome, and the expected compiled matched-template count is conserved for
each stylesheet.

## Claim boundary

This is evidence that FastXSLT's admitted use-last path produces the requested
recovery outcomes for these exact cases. It does not select general XSLT 1.0 or
2.0 compatibility, warning behavior, a host-visible conflict policy, or the
corresponding `b` variants. Those six cases request
`on-multiple-match=error` and expect `XTRE0540`; they remain explicit not-run
dispositions until the compiler can deliberately select and report that policy.

The 50-case apply-templates ledger now records 40 passes and 10 visible not-run
dispositions.
