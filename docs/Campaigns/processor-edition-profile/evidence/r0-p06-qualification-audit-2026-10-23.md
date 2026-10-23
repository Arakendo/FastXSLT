# R0 P06 qualification audit

Date: 2026-10-23. Base: `b4c92004`, plus the two guards and regression in this
tranche. Scope: approved R0 artifact/graph qualification, not feature admission.

## Finding and repair

Repository-wide inspection of `qualify_module`, stamp assignments and graph
entry points found two remaining Q01 routes in the older test-only global
constructor reference: `compile_with_single_include` and
`compile_with_single_import`. Both accepted an arbitrary compilation context,
lowered with the declaration-driven compiler and then stamped both artifacts.
Matching stamps therefore did not establish matching lowering semantics.

The new
`processor_edition_reference_global_constructor_linking_cannot_claim_selected_policy`
test failed before repair with `baseline reference must reject selected policy`.
Both helpers now call the existing `require_declaration_driven_lowering` guard
before dependency inspection/lowering. The test covers include/import, working
baseline compilation/execution, and refusal of both selected processor stamps
with exact `FXST1058`, Unsupported category and principal provenance.

This is Q01 within P06, not a new obligation or architecture decision. The
helpers are test-only; no production wrong-result claim follows from this probe.
Their current declaration-driven callers remain valid. No grammar was widened.

## Qualification route inventory

Paths below are relative to `crates/fastxslt/src/`.

| Owner / routes | Qualification boundary and audit disposition |
| --- | --- |
| `compile/stylesheet_preprocessing.rs`: context creation and `qualify_module` | Context fields are private; selected context construction is test-only. Final stamping is not semantic validation. Callers below must establish eligibility. |
| `compile/golden_stylesheet_experiment.rs`: initial program construction | Creates declaration-driven / modern-baseline artifacts. No declaration selects processor identity. |
| `compile/stylesheet_module_compiler.rs`: context-taking includes, single/multiple imports, mixed composition and document imports | Existing five entry guards plus the internal mixed-composition guard reject selected contexts before baseline lowering. The executable five-shape positive/negative matrix remains green. |
| Same owner: single-include legacy route, include merge, single/multiple import assembly and mixed merge | Shared `ensure_same_compilation_policy` checks processor and preprocessing independently. Mixed merge checks both supplied dependencies; multi-import assembly checks each dependency. Baseline entry wrappers cannot adopt a selected dependency's policy. |
| `runtime/resource_compiler/stylesheet_graph_context.rs` | Selected context returns `Some(Err(...))` before leaf, intermediate or root handling, never `None` that could trigger baseline fallback. Twelve context tests cover graph composition and qualification controls. |
| `runtime/resource_compiler.rs`: preprocessing reference | `for_reference` varies preprocessing only and retains declaration-driven processor policy. Its final qualification and fallback are not selected-processor admission. P08 must not reuse this route as though it selected an edition. |
| `compile/global_constructor_reference.rs`: include/import | Residual Q01 paths repaired in this tranche. Both refuse selected policy before baseline construction. |
| `compile/instruction_compiler/processor_edition_reference.rs`: single root-template lowering | Explicit edition/template callback and effective preprocessing precede final qualification; bounded shape checks remain in force. |
| `processor_edition_reference/matched_integration.rs` | Selected root/version, template, named-body, output and global callbacks precede qualification. Finalizers preserve stamps. This is still a test-only implementation; callback semantic completeness belongs to P01/P02/P04/P05, not a stamp check. |
| `processor_edition_reference/module_graph/mod.rs` | Selected principals are lowered with supplied global names and excluded validated dependency edges. Includes/imports/mixed graphs use shared checked mergers. Existing shape/dependency ceilings remain explicit. |
| `processor_edition_reference/include_chain.rs`: dependency shell | Preflight restricts the parent to exactly one validated include/import edge before baseline construction of the otherwise empty shell. No principal instruction may enter this baseline route. Shared mergers check the supplied dependency. |
| `processor_edition_reference/qualified_artifact.rs` | Wrapper validates both profile dimensions, identity repetition and bounded depth before shell composition. Shared program stamps remain checked independently after wrapper removal. |
| Test fixtures in `tests.rs`, `qualified_import_tests.rs`, `stylesheet_graph_context.rs` | Deliberate foreign/matching stamp fabrication is negative-test setup, not an eligible lowering entry. Preserve those controls without treating them as construction APIs. |

## Existing witnesses retained

- `processor_edition_reference_baseline_compiler_cannot_claim_selected_policy`:
  five baseline graph entry shapes with executable positive controls.
- `processor_edition_reference_shared_linker_preserves_or_rejects_processor_stamp`
  and `processor_edition_reference_existing_linker_refuses_mixed_preprocessing_artifacts`:
  independent processor/preprocessing mismatch rejection.
- `processor_edition_reference_artifacts_outlive_concurrent_compilation_documents`
  and `processor_edition_reference_failed_recompile_and_same_uri_replacement_preserve_old_artifact`:
  concurrent isolation, source release and failed replacement.
- Qualified-chain tests retain independent-dimension checks, transitive guards,
  bounded controls and cancelled final-link non-publication.
- Selected module-graph tests retain positioned include and import precedence,
  branching name resolution, supplied graph refusals, cycles and cancellation.

## Reproduction and validation

```powershell
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features preprocessing_context
cargo test -p fastxslt --all-features selected_policy_ -- --include-ignored
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

Initial replay without the corpus environment failed 21 fixture-loading tests;
after setting the documented root, the unmodified pre-repair campaign passed
227 probes and 12 context tests. The new regression then failed before the
guards, establishing the residual defect rather than mistaking an environment
failure for semantic evidence. Post-repair validation is recorded below.

Post-repair: 228 processor-policy probes and 24 selected-policy replay/control
tests passed. Repository verification passed formatting, Clippy, 2,290 workspace
tests (2,246 core + 22 native + 4 WASM + 18 worker), docs, local links, unsafe
surface and pinned conformance checks. The official WASM build passed; its
incremental cache reported a hard-link fallback to copying, not a build failure.
Logs are local `.workbench/r0-p06-{repro,probes,selected,verify,wasm}.log` files.

## Remaining boundary

This closes the current qualification inspection and repairs the demonstrated
residual Q01 routes. It does not close P06's whole promotion obligation:
non-test migration under P01 must retain these boundaries and rerun qualification
on the actual admitted entry. A full selected-profile corpus report remains P08.
No new exact match is claimed. Production remains the prior 2,504/3,173 (78.92%)
observation; strict-preprocessing baseline and selected-processor evidence remain
separate. No obligation count or semantic scope was enlarged.
