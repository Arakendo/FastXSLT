# Post-R0 selected comment integration

Date: 2026-10-23. Base commit: `b5935c8e` plus the source changes identified in
the [run manifest](post-r0-comment-run-identity-2026-10-23.json).
The maintainer explicitly approved the bounded comment slice proposed in the
[frontier reassessment](post-r0-frontier-reassessment-2026-10-23.md).
R0 remains closed at 9/9; its promotion report is unchanged.

## Implemented boundary

The crate-private selected legacy instruction dispatcher admits `xsl:comment`
with no attributes. A named private lowering helper compiles its body through
selected-policy sequence lowering and emits the existing `Xslt10CommentNode`.
There is no call into baseline body lowering, new runtime evaluator or parser.
The shared runtime owns text-only content conversion, comment lexical recovery,
result construction, work charging and serialization.

The body inherits selected policy, strict preprocessed content, existing lexical
bindings and depth guard. All preexisting expression and instruction admission
guards remain in force. Comment attributes, including the modern `select`
attribute, remain explicitly unsupported; this slice does not silently apply
modern semantics or claim complete legacy attribute recovery. Unsupported
availability introspection remains unsupported rather than gaining invented
answers. No public selector, default, acquisition authority or adapter changes.

## Corpus result

Corpus remains OASIS `XSLT-testsuite-04.ZIP`, SHA-256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
Both full reports conserve all 3,173 catalog cases under one fixed profile each.

| Measurement | R0 promotion baseline | After comment integration |
| --- | ---: | ---: |
| Selected legacy / strict exact outputs | 358 | **393 (+35), 12.39%** |
| Selected initialization failures | 2,785 | 2,750 |
| Selected successful executions | 375 | 410 |
| Selected execution failures | 10 | 10 |
| Selected output mismatches | 2 | 2 |
| Selected expected-error observations, separate from exact | 115 | 115 |
| Ordinary declaration-driven / modern exact outputs | 2,504 | **2,504 (78.92%), unchanged** |

The unchanged `Lotus/message_message11#1` now reaches output comparison and
matches under the existing XML comparator. The earlier first-refusal mapping
was deliberately only an estimate (32 plain comment start-tag occurrences),
not an upper bound or promise; the full replay is authoritative for the +35.

The selected mismatches remain `Lotus/whitespace_whitespace21#1` and
`Microsoft/Text__78309#1`. Its comparator limitation, policy/reference exclusions
and expected-error accounting remain visible. Do not add 35 to the ordinary
baseline: these are selected-profile gains, not progress to 80% on that lane.

- [Full selected report](post-r0-comment-selected-report-2026-10-23.txt)
- [Independent ordinary baseline report](post-r0-comment-baseline-report-2026-10-23.txt)
- [Source/toolchain/report identities](post-r0-comment-run-identity-2026-10-23.json)

## Focused evidence and gates

Three direct non-test-entry regression tests cover multiple cells:

- Declared versions 1/2/3, source-dependent content, strict comment-separated
  whitespace, disable-output-escaping text conversion, non-text result omission,
  double/trailing-hyphen recovery, message content exclusion and repeated reuse.
- Known-newer instruction fallback stays legacy; dormant/reached arity failure
  keeps its timing and provenance; an unbound variable remains a compile failure
  even under a dormant branch. Guarded expressions and attributes remain refused.
- Exact/one-less result-node budgets, cancellation triggered during execution,
  and successful reuse after failures through the same compiled artifact.

Initial test-authoring assumptions about empty-element spelling, an unadmitted
introspection expression and charge domains were corrected to the actual shared
contracts; no engine behavior was relaxed to satisfy those assumptions.

Final verification: **2,303 workspace tests** (2,259 core, 22 native, 4 WASM,
18 worker), **230 policy/reference probes**, **24 selected replay/control tests**,
fmt, Clippy, documentation/links, unsafe-surface and pinned corpus checks passed.
Official `wasm32-unknown-unknown` build passed. Environmental incremental-cache
hard-link warnings fell back to copying. Fixtures and upstream submodules are
unchanged. No performance claim is made by this semantic slice.

ADR-0004 inspection: the existing matched-integration unit remains a cohesive
selected-policy admission/lowering owner in the 1,001–2,000-line review band.
Comment lowering is a named helper beside message lowering; their common
dispatch avoids growing the main dispatcher beyond its Clippy limit. The new
focused tests are a private child of the selected-entry tests, in the existing
resource-compiler folder (five direct files); no new semantic layer was created.

## Reproduction and stopping point

```powershell
cargo test -p fastxslt --all-features selected_comment -- --nocapture
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features selected_policy_ -- --include-ignored
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10 -TraceCase message_message11
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

Approved slice complete. No new architectural blocker was exposed. Stop here
rather than admitting PI construction, descendant paths, broader copy selectors
or output encodings. A later slice starts from a newly selected failure and its
explicit scope; it does not reopen or rewrite the R0 promotion baseline.
