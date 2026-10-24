# B01-S3: single named root-descendant iteration

Ordinary bounded breadth, 2026-10-23, under the
[preimplementation declaration](../breadth-workflow.md). The unchanged
`Lotus/dflt_dflt04#1` refused `for-each select="//center"` at source span 425..457.
The selected iteration guard now admits exactly `//` plus one ASCII NCName,
using shared selection parsing/evaluation and selected-policy body lowering.
No new evaluator, ownership, namespace default, resource authority or expression
policy was introduced. Relative descendants, extra steps, wildcard/qualified
names, predicates, sorting, variables and resource functions remain excluded.

## Conserved replay

Same pinned OASIS archive and original 3,173-case denominator as
[S1](b01-s1-selected-processing-instructions-2026-10-23.md).

| Measurement | Before | After |
| --- | ---: | ---: |
| Selected legacy / strict exact | 469 | **482 (+13), 15.19%** |
| Successful executions | 498 | 511 |
| Initialization failures | 2,662 | 2,648 |
| Execution failures | 10 | 11 |
| Separate expected-error credits | 117 | 118 |
| Output mismatches | 2 | 2 |
| Ordinary declaration-driven / modern exact | 2,504 | **2,504 (78.92%)** |

The additional execution failure is explained, not an unexplained regression:
`Microsoft/AVTs__77562#1` previously refused its iteration selector, and now
reaches the existing invalid computed-name error `XTDE0820` at 249..279. It is
an expected-error catalog case. That credit is not included in the exact count.
The normalized non-XML subset counter rises 9 → 10; it is not added again to
the exact total. Exclusion counts and mismatch identities remain unchanged.

The motivating `dflt04` reaches its next existing guard at 613..708, the
`translate()` expression in another template. This is a downstream unsupported
family, not permission to expand this slice. Its original output is not credited.

- [Selected full report](post-r0-descendant-selected-report-2026-10-23.txt)
- [Independent ordinary report](post-r0-descendant-baseline-report-2026-10-23.txt)
- [Source/toolchain/report manifest](post-r0-descendant-run-identity-2026-10-23.json)

## Verification and stop

Three direct selected-entry tests pass across versions 1/2/3: root selection
from non-root focus, ordered results, unprefixed name namespace distinction,
empty selection, position/size and caller-focus restoration, whitespace-view
string values, excluded adjacent grammar with provenance, exact/one-less XPath
node visits, cancellation during charged traversal and post-failure reuse.

Full verification passes: **2,314 workspace tests** (2,270 core, 22 native,
4 WASM, 18 worker), fmt, Clippy, docs/links, unsafe-surface and pinned conformance
checks. The 230 processor-policy probes, 24 selected replay/control tests and
official WASM build pass. No unexplained regression or provisional blocker.

ADR-0004: a small existing-owner eligibility change, with a cohesive private
test module; the resource-compiler directory has eight direct files. No new
abstraction or extraction is needed for this slice.

Reproduce using `cargo test -p fastxslt --all-features selected_descendant_iteration`,
the S1 gate commands, and full selected measurement with `-TraceCase dflt_dflt04`.
For all failure identities, use `-TraceCase '#'` and capture the complete output
before filtering. S3 is complete: **B01 3/3; stop for maintainer review**.
