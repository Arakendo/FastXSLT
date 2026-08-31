# FastXSLT Contributor Instructions

## Project intent

FastXSLT is a Rust-native XSLT engine embedded by other applications, including
performance-sensitive ASP.NET services. Treat standards semantics, diagnostics,
host resource access, security limits, interop cost, and conformance evidence as
product concerns rather than incidental parser details.

The current source-of-truth design document is
`docs/Specifications/FastXSLT Software Design Document.md`. Update it or add an
ADR whenever code intentionally changes architecture, ownership, or a public
contract.

## Source of truth

- Read relevant accepted records in `docs/ADR/` before changing an established
  boundary or adding a cross-layer dependency.
- Read relevant records in `docs/Architectural Reviews/` when a question is
  unresolved, incubating, deferred, or reopened.
- Specifications describe current intended contracts within accepted ADRs.
- Plans sequence work. They do not override specifications or ADRs.
- Change Requests preserve a consumer's problem and requested boundary. They
  do not commit FastXSLT to the consumer's types, architecture, or schedule.
- Evidence records observations. It does not silently create a guarantee.
- A hand-authored fixture is useful evidence, but it is not a conformance claim.

## Architecture boundaries

- Keep XML parsing mechanics replaceable. FastXSLT owns XDM, XPath, XSLT, and
  diagnostic semantics even when a third-party parser supplies XML events or a
  tree.
- Keep engine semantics host-neutral. ASP.NET, .NET interop, CLI, WASM, and other
  consumers are adapters over the same compiled-stylesheet and transformation
  contracts, not alternate semantic engines.
- Keep stylesheet compilation separate from transformation so compiled
  stylesheets can eventually be reused safely.
- Keep compiled artifacts limited to stylesheet-derived static state. Put source
  documents, parameters, messages, clocks, resolver state, budgets, and other
  invocation state in the transformation runtime.
- Keep host-controlled I/O behind explicit resolver interfaces. Engine internals
  must not silently read the filesystem, access the network, expand external
  entities, or inherit ambient process authority.
- Keep resource identity distinct from filenames, display names, host paths,
  content fingerprints, and retained byte storage. Import adapters may load
  files; engine execution consumes qualified resources or a sealed snapshot.
- Follow ADR-0002: after import, compilation and execution are memory-resident by
  default. Do not retain or reopen source file handles, memory-map source files,
  emit intermediate artifacts to disk, spill to temporary files, or introduce a
  hidden disk cache. A disk-backed mechanism requires explicit host authority
  and a deliberate ADR revision or supersession.
- Prefer compile-once and transform-many boundaries. A convenience single
  transform API should use the same semantics as a batch of one rather than
  developing a separate file-oriented execution path.
- Follow ADR-0005: a transform set contains independent requests with no start,
  execution, or completion-order contract. Correlate results by logical identity,
  keep sibling results invisible, and leave dependent workflow stages and
  result-to-resource admission to the host.
- Follow AR-0009's incubation guardrails for prepared inputs. Reusable prepared
  state is immutable and source-derived; never store invocation state in it,
  equate content hashes with document identity, eagerly parse every snapshot
  entry by default, or introduce global/cross-snapshot caching without evidence.
- Follow ADR-0012 for exact `xsl:strip-space elements="*"`: compose immutable
  prepared XDM with a private invocation-owned visibility view, preserve visible
  node identity and provenance, route every source-semantic consumer through
  effective relationships, retain the complete safe derivation as a test
  oracle, and do not infer broader whitespace rules, a public provider trait,
  or cross-invocation view retention.
- Follow ADR-0013 for document-rooted match paths: lazily build only bounded
  invocation-owned membership keyed by the current compiled template, preserve
  the complete charged evaluator as fallback and differential oracle, and do
  not share membership across invocations, sources, snapshots, workers, or
  generations.
- Preserve source locations and structured diagnostics across XML parsing,
  XPath parsing/evaluation, stylesheet compilation, lowering, optimization, and
  execution.
- Keep the semantic transformation result distinct from serialization into
  text, bytes, or an output sink, even if an early implementation combines them.
- Follow AR-0007's deferred guardrails. A first tree evaluator may be concrete,
  but do not spread an unnecessary assumption that every source is permanently
  a fully materialized random-access tree. Depend on the semantic navigation
  actually required at each layer, keep representation-specific access inside
  its owner, and do not manufacture generalized provider traits before another
  strategy or measured seam needs them.
- Keep observability explicitly supplied and semantically inert. Do not use
  ambient global subscribers as the only way to inspect engine work.
- Do not introduce a second execution backend without a parity strategy against
  the reference semantics.
- Do not describe architectural streaming optionality, event-fed parsing, or
  bounded subtree buffering as XSLT streaming conformance. ADR-0007 deliberately
  excludes that claim; any future implementation requires renewed review and
  dedicated conformance evidence.
- Do not split logical layers into crates until dependency direction, independent
  reuse, or release pressure makes the boundary valuable.
- Follow ADR-0003. Do not add `unsafe` code merely because tests pass. The
  engine and ordinary workspace crates forbid it. ADR-0008 admits only the
  native .NET workbench's reviewed export and bounded buffer-copy surface; any
  other exception needs its own
  accepted ADR covering necessity, rejected safe alternatives, safety contract,
  containment, safe reference behavior where practical, specialized tool
  evidence, measured benefit, exact surface, and removal criteria.

## Design habits

- Prefer a small vertical transform slice over broad placeholder abstractions.
- Tie every public abstraction to a real caller, golden case, conformance case,
  or host integration requirement.
- Measure consumer-visible latency and throughput across the host boundary;
  Rust-only microbenchmarks do not establish ASP.NET application performance.
- Measure preload, parse, compile, warm execution, result transfer, and peak
  retained memory separately. “In memory” is a strategy to verify, not a
  substitute for an end-to-end profile.
- Include handle-release tests: after a host adapter imports a file, callers
  must be able to rename, replace, or remove the original without invalidating
  the sealed snapshot or an in-flight transform.
- Distinguish unsupported behavior from invalid input in diagnostics.
- Distinguish a reportable semantic outcome from an operation failure. A host
  must not have to parse display strings to tell unsupported behavior, denied
  authority, exhausted budgets, invalid input, and internal failure apart.
- Keep inspection and explainability surfaces read-only and semantic. Do not
  make a private AST, IR, arena layout, cache key, or optimizer detail public
  merely so a host can diagnose compiled or executing work.
- Make resource limits and fallback behavior explicit and testable.
- Record the exact standards edition and test-suite version behind every
  conformance number.
- Do not let a convenient private Rust type silently settle a decision that the
  SDD lists as open. Open choices affecting ownership, public boundaries,
  authority, concurrency, replaceability, or conformance require review and an
  ADR before becoming contracts.
- Follow ADR-0004 when source units accumulate size or responsibility pressure.
  Line count triggers review; split only along named ownership/responsibility
  seams, prefer private modules, inspect post-extraction coupling, and preserve
  standards behavior, diagnostic provenance, resource/batch contracts,
  performance baselines, ABI behavior, and unsafe-code invariants.
- Preserve imported fixture provenance, license information, and byte-level
  integrity where the upstream suite requires it.
- Treat `vendor/qt3tests` and `vendor/xslt30-test` as immutable upstream
  submodules. Do not edit expected results in place, copy their content into the
  MIT-licensed crate, or move a gitlink without updating the corpus provenance
  record and reviewing the upstream/license delta. Put harness selection,
  exclusions, classifications, and corrections in first-party overlays.
- FastXSLT is MIT licensed. Record dependency, fixture, and copied-code licenses
  before admission, and do not introduce terms that prevent distributing the
  FastXSLT library under its declared MIT license without explicit maintainer
  review.

## Validation

Run these gates after Rust changes:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
```

`scripts/verify.ps1` runs the same local gate set and checks local Markdown link
targets. New public behavior needs at least one focused test and one
representative golden or corpus case. Changes to documentation navigation must
keep relative links valid.

## Current workspace shape

- `crates/fastxslt` -- public facade and initially private engine layers
- `crates/fastxslt-dotnet-workbench` -- unpublished ADR-0008 native experiment;
  its exact unsafe surface is enforced by `scripts/verify.ps1`
- `crates/fastxslt-worker` -- unpublished isolated-process host experiment
- `corpus/golden` -- small reviewed source/stylesheet/expected triples
- `vendor/qt3tests` and `vendor/xslt30-test` -- pinned upstream W3C suites
- `docs/Specifications` -- current intended architecture and contracts
- `docs/ADR` -- accepted architectural decisions
- `docs/Architectural Reviews` -- open questions, evidence, and dispositions
- `docs/Plans` -- executable work and milestone sequencing
- `docs/Change Requests` -- incoming consumer needs and requested boundaries
- `docs/Evidence` -- reproducible observations and review records
