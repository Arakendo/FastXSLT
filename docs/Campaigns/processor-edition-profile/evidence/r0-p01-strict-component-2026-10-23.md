# R0 P01: non-test strict preprocessing component

Date: 2026-10-23. Second approved interval, P01 in progress; **0/3 tranches
complete, 0/9 whole obligations closed**. This dependency extraction does not
count as completion of the non-test selected compiler migration.

## Change and boundary

The selected compiler depended on test-only strict preprocessing and XDM child
filtering. A private `strict_content::prepare` now builds with the ordinary
workbench engine and official WASM target. It applies the existing strict rules
without carrying modern-only preprocessing into this component. The XDM helper
remains semantic-neutral, immutable, charged and authority-free.

Selected legacy matched-module lowering now uses the component, including its
existing graph callers. The old `strict_stylesheet_view` / general visibility
implementation stays test-only as the differential oracle. Modern reference and
ordinary production entry points are unchanged. No admitted feature was added.

The full selected entry remains test-only. Two narrow, documented non-test
`expect(dead_code)` annotations mark pending graph wiring/failure projection;
remove them when P01 supplies the non-test caller. They do not waive Clippy or
authorize profile promotion. This is deliberately an intermediate dependency
checkpoint, not a claim that moving a helper establishes a supported profile.

## Evidence

The differential test covers comment/PI-separated fragments, inherited/reset
`xml:space`, explicit `xsl:text`, embedded-subtree isolation, original identity,
shared immutable storage and provenance. It compares visible relationships over
all original nodes and identical XDM charges. Every budget from zero through the
exact required budget is compared against the reference, including failures;
pre-cancellation and original-source preservation are also checked.

- Full verification: **2,292 tests passed**, formatting, Clippy, docs/links,
  unsafe-surface and pinned conformance checks passed.
- Processor-policy suite: **229 passed**, including ignored corpus probes with
  the corpus root configured. An initial invocation omitted that environment
  variable; its 21 corpus-setup failures disappeared with the documented root.
- Official WASM build passed (incremental cache used its copy fallback).
- Full selected/strict OASIS replay: **358/3,173 exact, two mismatches**. Every
  captured report line matches the [before-migration report](r0-selected-profile-full-report-2026-10-23.txt),
  including diagnostic partitions and exclusions, not just the numerator.
  Production coverage remains the historical 2,504/3,173, not newly replayed here.

Candidate: dirty working tree over `b4c92004dcf6778135d3aff4512c410c83c9d00d`,
same Rust 1.95.0 toolchain and corpus as the preceding report. New component
SHA-256: `220E760471974EA3A432909730D0BC4DE9DFF914B70768964F7643ACD4E87EEC`.
Local full-run log SHA-256:
`09E0C61F005821B76A487C8F9AD48C758D963CC608EB51C0A0D02C8BFE993A67`.
These are checkpoint identities, not a clean release build or a replacement for
the earlier run's complete source manifest.

```powershell
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features non_test_strict_component
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10
```

Next: separate selected legacy lowering dependencies from test probes/modern
witnesses, supply the non-test graph entry and remove the transitional wiring
annotations. Preserve the frozen admission guards and retained oracle. No new
decision or provisional blocker was found in this component.
