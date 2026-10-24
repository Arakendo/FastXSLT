# B07-S3: terminal named-attribute value paths

Scope declared before implementation in the [workflow](../breadth-workflow.md).
Fresh `select01` refusal at 409..448 reads `/doc/a/b/@attr` inside iteration.
Admit a final unprefixed named attribute after named child steps, optionally
rooted or explicitly relative. Shared legacy value lowering selects the first
node in document order and constructs text. This is no attribute-construction,
ownership, authority or public-contract change.

Tests pin nested focus versus document root, first matching attribute when an
earlier element has only a namespaced attribute, empty paths, escaping, declared
versions and exact/one-less result budgets, charged XPath cancellation and reuse.
Qualified names, attribute wildcards, predicates, descendants and unions remain
guarded. The S2 exclusion test now uses an attribute wildcard rather than the
newly admitted named attribute.

Selected **526 → 527/3,173 (+1)**; `select01` exact. Executions 559 → 560;
initialization failures 2,600 → 2,599. Mismatches 2, execution failures 11 and
expected-error credits 118 unchanged. Ordinary remains **2,504/3,173** after
independent replay. All repository gates (2,334 workspace tests), 230 policy
probes, 24 selected replay/control tests and official WASM build pass.
[Selected](b07-s3-selected-report-2026-10-23.txt),
[ordinary](b07-s3-baseline-report-2026-10-23.txt),
[identity](b07-s3-run-identity-2026-10-23.json).
