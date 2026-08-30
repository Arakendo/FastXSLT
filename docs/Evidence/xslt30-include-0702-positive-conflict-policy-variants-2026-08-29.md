# XSLT30 `include-0702` Positive Conflict-Policy Variants

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Cases: `include-0702a`, `include-0702c`
- Shared principal and dependencies: `include-0701.xsl` plus four secondary
  modules
- Shared source/result: `include-07.xml` and `include-0702c.out`

## Conserved metadata

The two positive cases intentionally exercise the same stylesheet graph and
expected XML, but they do not carry the same suite meaning:

| Case | Standards dependency | Multiple-match dependency | Disposition |
| --- | --- | --- | --- |
| `include-0702a` | `XSLT10 XSLT20` | `recover` | passed |
| `include-0702c` | `XSLT30+` | not specified | passed |

Focused tests verify those dependencies directly from the pinned test-set
metadata before executing each case. Both compile the sealed five-module graph,
retain four lower-precedence and six principal-precedence rules, select the
later same-precedence `title` rule, and produce an XML-equivalent result.

## Claim boundary

This admits the suite's explicit recovery variant and its XSLT 3.0+ positive
variant on one exact private graph. It does not expose a general or public
multiple-match policy. In particular, `include-0702b` remains
`harness-unsupported / not-run`: that case requests `on-multiple-match=error`
and expects `XTRE0540`, which FastXSLT cannot yet request through compilation or
invocation policy. Reusing the recovery path for that case would be a false
pass, not conformance evidence.

The conserved 16-case include denominator now has 12 explicit passes and four
visible not-run dispositions.

## Subsequent disposition

Later on 2026-08-29, the private invocation-local error policy admitted
`include-0702b` without reusing the recovery path. See
[the dedicated error-policy evidence](xslt30-include-0702b-multiple-match-error-2026-08-29.md).
The current denominator is therefore 14 passes and two DTD-dependent not-run
cases; the figures above preserve this record's earlier positive-variant
checkpoint.
