# XSLT30 Conflict-Resolution Multiple-Match Error Policy

Date: 2026-08-29

## Scope

This tranche executes the six error-policy variants in the pinned XSLT30
apply-templates test set:

- `conflict-resolution-0102b`
- `conflict-resolution-0104b`
- `conflict-resolution-0108b`
- `conflict-resolution-0110b`
- `conflict-resolution-0401b`
- `conflict-resolution-1202b`

Each case declares `on-multiple-match=error` and expects the native suite error
pattern `XTRE0540`.

## Executable behavior

The private transform-set builder now selects one of two invocation-local
policies: use the later declaration or reject an ambiguous selection. The
default remains the already-evidenced use-last path. The error path separates
the semantic rank `(import precedence, priority)` from declaration order and
reports concrete dynamic error `XTDE0540` when more than one applicable rule
occupies the highest eligible semantic rank.

The check applies both to ordinary template dispatch and to continuation by
`xsl:next-match`. The latter distinction is exercised by `1202b`: its initial
priority-5 and priority-4 rules are unique, but continuation eventually reaches
two priority-3 wildcard rules and must fail there. A supplemental execution of
`conflict-resolution-0101` under the error policy proves that a lower-ranked
tie does not preempt a unique higher-ranked rule.

Failures retain the request identity and selected-template source location and
use the structured `Invalid` category. The corpus assertion compares the suite
pattern `XTRE0540` with FastXSLT's concrete `XTDE0540`; it does not rewrite the
upstream expected result.

## Result

All six native error cases pass. The complete apply-templates denominator is
now 47 selected/passed and 3 explicitly not-run cases out of 50. All eight
native error assertion shapes in that set are executed.

## Claim boundary

This is private reference-path and corpus evidence. It does not expose a public
Rust or host-adapter policy, select a general XSLT 1.0/2.0 compatibility mode,
or, within this tranche, admit the separate `include-0702b` case. That case was
subsequently admitted through the same private policy seam. Warning delivery
for recovery mode remains outside this evidence.
