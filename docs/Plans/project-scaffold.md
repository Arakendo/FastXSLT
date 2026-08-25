# Project Scaffold

| Field | Value |
| --- | --- |
| Status | Complete |
| Opened | 2026-08-25 |
| Completed | 2026-08-25 |

## Objective

Create a clean Rust work environment for FastXSLT by adapting the useful
governance, evidence, and verification patterns observed in the Tokimu peer
repository and its embedded Weaver XSLT project.

## Deliverables

- [x] Cargo workspace and stable toolchain configuration.
- [x] One publish-disabled library crate with private logical layer placeholders.
- [x] Formatting, Clippy, tests, rustdoc, local script, editor, and CI gates.
- [x] Documentation index separating specifications, ADRs, ARs, plans, evidence,
  notes, and corpus policy.
- [x] Initial SDD, accepted starting-structure ADR, and standards-profile AR.
- [x] Seed golden transform with an explicit non-conformance disclaimer.
- [x] Peer-review evidence explaining which patterns were adopted or deferred.

## Acceptance criteria

- [x] `cargo fmt --all --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passes.
- [x] `cargo test --workspace --all-features` passes.
- [x] `cargo doc --workspace --no-deps` passes with warnings denied.
- [x] Local Markdown link targets pass the repository checker.
- [x] A new contributor can identify binding decisions and unresolved questions
  from the repository root.
