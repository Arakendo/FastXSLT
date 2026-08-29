# Stylesheet-Module Assembly Decomposition Review

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Governing decision | ADR-0004 |
| Parent immediately before review | 1,364 physical lines |
| Single-document compiler after extraction | 1,135 physical lines |
| Existing instruction compiler | 775 physical lines |
| New module assembly owner | 176 physical lines |
| New whole-program validation owner | 80 physical lines |
| Disposition | Reopening trigger discharged by named private owners |

## Trigger

The first executable `xsl:include` case introduced top-level dependency
discovery, secondary simplified-stylesheet compilation, module merge rules, and
post-merge validation. The parent stylesheet compiler reached 1,364 lines. This
fired both reopening conditions retained by the earlier runtime/compiler
review: it crossed 1,200 lines and acquired a demonstrably independent
top-level declaration/validation phase.

## Ownership after extraction

`golden_stylesheet_experiment.rs` remains the single-document composition
owner. It recognizes the stylesheet root, compiles top-level declarations and
templates, provides shared structural diagnostics, and calls instruction
lowering.

`stylesheet_module_compiler.rs` owns the deliberately narrow dependency slice:
discovering one `xsl:include`, validating its declaration shape, compiling one
simplified secondary stylesheet, and merging its existing semantic program
with the principal program. Its duplicate and precedence checks fail explicitly
where the private slice lacks sufficient standards semantics.

`stylesheet_validation.rs` owns whole-program named-template reference and
argument validation. Running it after module assembly no longer requires the
module owner to absorb recursive instruction validation.

The runtime resource compiler remains the adapter between these language
semantics and AR-0014's sealed snapshot resolver. It resolves and parses the
secondary resource; neither compiler child imports snapshot, filesystem,
network, host, or runtime policy.

## Dependency direction and coupling

The single-document compiler calls instruction lowering and whole-program
validation. Module assembly calls the single-document compiler for the
principal module, instruction lowering for the simplified secondary root, and
whole-program validation after merge. The children exchange the existing
private `StylesheetProgram`; they do not introduce callbacks, a broad mutable
compiler context, a public module graph, a second semantic backend, or a crate
boundary.

## Conservation and claim boundary

The extraction preserves the passing XSLT30 `include-0401` result and the
structured missing-secondary outcome. It preserves resource/source identity,
diagnostic locations, snapshot authority, global visibility, execution and
serialization paths, public Rust/API surfaces, and the workspace unsafe-code
policy.

The structure does not admit recursive dependencies, general include
precedence, `xsl:import`, fragments, catalogs, or live resource acquisition.
Those semantics must expand this owner only with corpus evidence and AR-0014's
bounded dependency policy.

## Validation

`scripts/verify.ps1` passes the enforced unsafe-surface check, formatting,
workspace Clippy with warnings denied, all-feature workspace tests, local
Markdown-link validation across 154 files, conformance-source cleanliness and
inventory, XSLT30 metadata inventory, and workspace documentation. The engine
result is 214 passed and 7 ignored manual probes; the native workbench adds 7
passes.
