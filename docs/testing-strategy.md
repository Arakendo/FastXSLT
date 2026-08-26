# FastXSLT Testing Strategy

FastXSLT separates quick implementation feedback from architectural and
standards evidence. A higher tier complements lower tiers; it does not replace
them.

| Tier | Purpose | Typical location |
| --- | --- | --- |
| Unit | Lexer, parser, data-model, matching, conversion, and diagnostic rules | crate modules |
| Golden | Small source + stylesheet + expected-result vertical cases | `corpus/golden/` |
| Conformance | Versioned upstream standards suites with explicit selection | documented external corpus |
| Differential | Compare supported semantics with named reference processors | harness-specific |
| Integration | Public API, resolver, serialization, ASP.NET/.NET, CLI, WASM, or FFI boundaries | workspace and consumer tests |
| Property/fuzz | Parser safety, invariants, round trips, and adversarial inputs | focused fuzz targets |
| Benchmark | Compile, warm transform, cold transform, allocation, and scale behavior | benchmark harness |

## Comparison rules

XML serialization is not generally safe to compare as raw text. Each golden or
conformance case must declare whether the assertion concerns exact text, parsed
tree structure, nodes and values, messages, diagnostics, or an expected error.
Namespace nodes, prefixes, attribute order, whitespace, encodings, and output
method rules need deliberate comparison behavior.

## Conformance reporting

Every report must record:

- standards edition and claimed profile;
- upstream suite name, version or commit, and license;
- acquisition and integrity procedure;
- selected, excluded, unsupported, passed, failed, and harness-error counts;
- FastXSLT revision, features, target, and toolchain;
- reference processor and version for differential results;
- known harness limitations.

Unsupported tests must not be counted as passed or silently omitted.

## Admitted conformance sources

The repository pins the W3C QT3 and XSLT 3.0 suites as Git submodules under
`vendor/`. Their exact revisions and licensing boundary are recorded in
[W3C Test Suite Provenance](Corpus/w3c-test-suites.md).

The suites are inputs, not executable FastXSLT tests by themselves. The first
private XSLT30 slice resolves one explicit overlay selection and its upstream
environment, stylesheet, and XML assertion. A broader harness must parse all
relevant dependency and environment metadata, retain the
unaltered case identity, classify cases against the accepted AR-0001 profile,
and distinguish unsupported behavior from harness failure. Local selection and
classification belong outside the submodules.

## Resource and batch testing

Resource-snapshot tests must cover qualified identity, same-name resources,
missing and denied references, path traversal, entry/per-entry/aggregate limits,
duplicate physical content under distinct identities, snapshot immutability, and
replacement without mutating in-flight work.

Filesystem-adapter tests must prove handles are released before sealing. On
Windows, rename, replace, and removal of imported files must succeed while the
sealed snapshot remains usable. Tests must also verify that diagnostic
provenance, missing dynamic references, and warm transforms never trigger a path
reopen or implicit temporary/cache file.

Under ADR-0005, batch tests cover independent unordered requests rather than
dependency graphs. Randomized acquisition/start/completion order must preserve
deterministic result association, shared compiled stylesheets, isolated
parameters/dynamic context, failure collection versus fail-fast policy,
cancellation, partial output, concurrency, and a batch of one matching the
single-transform convenience API. Stage tests must prove sibling results are not
visible and become resources only after the host explicitly admits them into a
later snapshot.

## Performance reporting

Benchmarks run only after correctness checks for the measured cases. Record the
input sizes, stylesheet features, compile reuse, warmup, sample method, hardware,
OS, Rust toolchain, allocator when relevant, and comparison revision. Report
distributions or uncertainty rather than only a best observation.

Embedded-host benchmarks must measure from the consumer's call boundary through
result consumption. For ASP.NET this includes managed/native or process
marshaling, cancellation, stylesheet caching, execution, result transfer, and
serialization where used. Rust microbenchmarks remain useful diagnostics but do
not substitute for consumer-visible latency and throughput.

Volume benchmarks must report resource preload, snapshot sealing, source parse,
stylesheet compile, cold execution, warm execution, cache reuse, peak memory,
and total batch throughput separately. Compare against a file-oriented baseline
and a warmed filesystem-cache baseline before attributing gains to memory
retention.

## Unsafe-code exception verification

ADR-0003 keeps first-party unsafe code forbidden by default. If a later ADR
admits a narrow exception, ordinary tests remain only one evidence tier. The
exception's verification matrix must identify which invariants are inspected by
Miri, sanitizers, fuzzing, property tests, concurrency model checking,
platform/ABI tests, and stress tests, including any tool or target gaps.

Unsafe optimizations retain a safe reference implementation whenever practical
and run semantic differential tests across successful results, structured
diagnostics, cancellation, limits, malformed inputs, and concurrency. Benchmarks
run only after parity and safety-focused gates pass and must show that the
benefit remains material at the consuming-application boundary.
