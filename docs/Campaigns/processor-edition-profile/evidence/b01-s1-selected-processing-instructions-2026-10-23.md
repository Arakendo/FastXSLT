# B01-S1: selected static-target processing instructions

Date: 2026-10-23. Ordinary breadth under the maintainer-approved
[workflow and preimplementation scope](../breadth-workflow.md), not a promotion
or modification of R0. Source/toolchain identity is preserved in the
[run manifest](post-r0-pi-run-identity-2026-10-23.json).

## Scope and implementation

The unchanged witness `Lotus/message_message12#1` initially failed selected
initialization with unsupported `FXAR0031` at its PI constructor. Selected legacy
lowering now accepts `xsl:processing-instruction` with only a `name` attribute,
whose value is a static ASCII NCName other than case-insensitive `xml`.
Its body uses the existing selected-policy compiler, and execution uses the
existing shared `Xslt10ProcessingInstructionNode` runtime. PI text conversion,
`?>` recovery, messages, controls and serialization retain their existing owners.

Dynamic/invalid/qualified targets, additional attributes, new expression grammar
and adjacent constructors remain outside this slice. Missing required `name`
uses the existing structured invalid-input diagnostic; unsupported facilities
are not used as fallback opportunities. No fixture, comparator, default, public
surface, authority model or artifact qualification changed.

## Full conserved replay

OASIS `XSLT-testsuite-04.ZIP`, SHA-256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`;
all 3,173 original cases, fixed selected legacy / strict preprocessing profile.

| Measurement | Before | After |
| --- | ---: | ---: |
| Selected exact XML comparisons | 449 | **466 (+17), 14.69%** |
| Successful executions | 477 | 495 |
| Initialization failures | 2,683 | 2,665 |
| Execution failures | 10 | 10 |
| Output mismatches | 2 | 2 |
| Serialization-layout exclusions | 11 | 12 |
| Expected-error credits, separate from exact | 115 | 117 |
| Ordinary declaration-driven / modern exact | 2,504 | **2,504 (78.92%)** |

Eighteen additional successful executions yield 17 exact comparisons and one
additional layout exclusion. Unsupported `FXAR0031` initialization drops by 20;
invalid `FXST0008` increases by two, exposing required-attribute validation.
The two additional expected-error credits are separate, not exact-output gains.
No comparison rules changed. Mismatch identities remain `whitespace_whitespace21`
and `Text__78309`; no new mismatch or execution-failure regression is observed.
The named `message12` witness now passes the unchanged XML comparison.

- [Selected report](post-r0-pi-selected-report-2026-10-23.txt)
- [Independent ordinary report](post-r0-pi-baseline-report-2026-10-23.txt)
- [Source/toolchain/report identity](post-r0-pi-run-identity-2026-10-23.json)

## Verification and stopping condition

Three focused selected-entry tests pass: declared versions 1/2/3, strict
preprocessing, dynamic text content, PI recovery, message conversion, selected
fallback/introspection, deferred versus static errors, excluded target/attribute
guards, exact/one-less result-node budgets, active cancellation and artifact reuse.

Repository verification passes: fmt, Clippy, **2,310 workspace tests** (2,266
core, 22 native, 4 WASM, 18 worker), docs, Markdown links, unsafe-surface checks
and pinned conformance checks. The **230** processor-policy probes and **24**
selected replay/control tests pass separately. Official `wasm32-unknown-unknown`
build passes. Incremental-cache hard-link warnings used the existing copy fallback.

ADR-0004: the helper remains inside selected instruction lowering; focused tests
have a private sibling file. No new runtime responsibility or cross-layer owner
was introduced. B01-S1 is complete, **1/3** semantic slices. No provisional
blocker or approval request is needed for a subsequent eligible bounded slice.
The declared exclusions remain; this completion does not widen them.

```powershell
cargo test -p fastxslt --all-features selected_pi_ -- --nocapture
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features selected_policy_ -- --include-ignored
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10 -TraceCase message_message12
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```
