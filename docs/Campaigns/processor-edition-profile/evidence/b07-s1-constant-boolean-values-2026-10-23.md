# B07-S1: constant Boolean values

Fresh full selected trace at B07 start: 513 exact. Unique-filename, valid-span
triage counted 512 value-of refusals, 128 for-each and 32 copy-of. Ambiguous files
and unusable spans were excluded; these are ranking hints, not gains or a new
denominator. `boolean01`, `02`, `04`, `09` expose constant Boolean value guards.

Scope: bounded true/false, and/or, not/boolean and grouping trees only. Reuse the
existing selected parser and shared Boolean IR/text-result owner; an IR-kind
admission check excludes dynamic leaves without changing their error timing.
No new evaluator. Parser length/depth limits remain in place. Tests pin truth
tables, precedence, versions, excluded dynamic/conversion grammar and controls.
Lint pressure was addressed by keeping value eligibility in the Boolean owner,
not growing the large value-of function or suppressing the lint.

Selected **513 → 525/3,173 (+12)**; executions 546 → 558; initialization failures
2,613 → 2,601. Mismatches 2, execution failures 11, expected-error credits 118
unchanged. Ordinary **2,504/3,173**, independently replayed. All gates (2,332
workspace tests) and official WASM build pass. No blockers.
[Selected](b07-s1-selected-report-2026-10-23.txt),
[ordinary](b07-s1-baseline-report-2026-10-23.txt),
[identity](b07-s1-run-identity-2026-10-23.json).
