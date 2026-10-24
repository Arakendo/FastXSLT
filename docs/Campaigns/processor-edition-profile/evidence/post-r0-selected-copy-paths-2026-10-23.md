# Post-R0 named element-copy paths

Date: 2026-10-23. Maintainer approved the bounded scope proposed in the
[post-comment reassessment](post-r0-frontier-reassessment-2026-10-23.md#after-comments-proposed-named-element-copy-paths).
This is separate breadth after R0 and the comment slice, not a reopened ledger.
Source base is `b5935c8e` plus the changes identified in the
[run manifest](post-r0-copy-run-identity-2026-10-23.json).

## Change and invariants

Selected `xsl:copy-of` now admits nonempty slash-separated ASCII child names,
optionally rooted with a single leading slash. Surrounding expression whitespace
is trimmed for eligibility. The existing shared copy compiler still parses the
path, and selected lowering still requires a `CopyOfLocationPath` result and
explicitly supplies legacy recovery. No new parser, evaluator, resource access,
ordering policy or result representation was added.

Root-only `/`, `//`, empty/trailing steps, wildcard and qualified names,
predicates, attributes, namespace axes, variables, functions, unions and parent/
context steps remain refused. Iteration selectors and other instruction guards
are unchanged. This is not general XPath admission or a public profile selector.

## Conserved replay

OASIS `XSLT-testsuite-04.ZIP`, SHA-256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`,
unchanged 3,173-case denominator and original comparisons:

| Measurement | After comments | After named copy paths |
| --- | ---: | ---: |
| Selected legacy / strict exact | 393 | **449 (+56), 14.15%** |
| Selected successful executions | 410 | 477 (+67) |
| Selected initialization failures | 2,750 | 2,683 |
| Selected execution failures | 10 | 10 |
| Selected output mismatches | 2 | 2 |
| Selected expected-error observations, separate | 115 | 115 |
| Selected serialization-layout exclusions | 3 | 11 |
| Selected host-parser-policy exclusions | 4 | 6 |
| Selected unusable-reference exclusions | 5 | 6 |
| Ordinary declaration-driven / modern exact | 2,504 | **2,504 (78.92%), unchanged** |

The 11 newly successful executions not credited as exact remain in the existing
exclusion classifications: eight layout, two parser-policy and one unusable
reference. No classifier, expected result, denominator or per-case profile
selection changed. Successful execution alone earns no pass.

`Microsoft/Copying_Copy-of_CopyOfFromDocumentRoot#1` now reaches comparison and
passes unchanged, selecting `/xslTutorial`. The selected mismatch identities
remain `Lotus/whitespace_whitespace21#1` and `Microsoft/Text__78309#1`.
The 56 gains must not be added to the ordinary baseline. The R0 promotion
baseline remains preserved at 358 exact, and the comment checkpoint at 393.

- [Full selected report](post-r0-copy-selected-report-2026-10-23.txt)
- [Independent ordinary baseline report](post-r0-copy-baseline-report-2026-10-23.txt)
- [Source/toolchain/report identity](post-r0-copy-run-identity-2026-10-23.json)

## Validation

Four direct selected-entry regression tests prove:

- absolute versus relative paths from non-root focus, ordered sibling matches,
  empty selection and surrounding whitespace across declared versions 1/2/3;
- copied subtree namespace and attribute retention, plus preserve/strip behavior
  against the same immutable prepared source;
- explicit refusal and principal-resource provenance for excluded families;
- exact/one-less result-node budgets, in-execution cancellation and post-failure
  reuse of the same compiled artifact.

The first complete run caught a retained test expecting `num/child` to be
unsupported under the previous scope. It now explicitly accepts relative/rooted
named child paths while testing `num//child` remains refused. This is a reviewed
admission-boundary change, not a relaxed semantic oracle or corpus fixture edit.

Final repository verification passed: **2,307 workspace tests** (2,263 core,
22 native, 4 WASM, 18 worker), **230 policy/reference probes**, **24 selected
replay/control tests**, fmt, Clippy, docs/links, unsafe-surface and pinned corpus
checks. Official WASM build passed. Environmental hard-link cache warnings use
the existing copy fallback. No benchmark claim follows from these tests.

ADR-0004: this is a small eligibility change inside the existing selected-copy
owner, not a new responsibility. Its focused tests are a named private child of
the selected-entry suite; the resource-compiler directory has six direct files.
No further decomposition is justified by this slice.

## Reproduction and stop

```powershell
cargo test -p fastxslt --all-features selected_copy_path -- --nocapture
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features selected_policy_ -- --include-ignored
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10 -TraceCase CopyOfFromDocumentRoot
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

Approved slice complete; no new architectural blocker. Stop before wildcard,
descendant, predicate, atomic or cross-document copy selection. Reassess a named
failure before choosing the next bounded slice. Defaults and public APIs remain
unchanged; this does not reopen R0 or establish XSLT 1.0 conformance.
