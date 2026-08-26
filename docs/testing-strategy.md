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

## Corpus purposes

Conformance, adversarial, and performance corpora answer different questions
and must not share an unexplained denominator:

| Corpus family | Primary question | Must not imply |
| --- | --- | --- |
| Conformance | Is behavior correct for a named standards edition and selected feature profile? | Support for excluded/unsupported cases or security under load |
| Adversarial | Does hostile-but-bounded work terminate through the expected structural/work control? | Standards conformance or a production budget default |
| Performance | What does a correctness-gated workload cost under a recorded configuration? | Correctness outside the cases or end-to-end host speed from a Rust-only number |

An upstream suite may inspire an adversarial regression, but copied/minimized or
generated fixtures need their own provenance and cease to be unmodified upstream
conformance cases. A benchmark may reuse a correct fixture, but its sampling,
warmup, scaling, and host configuration belong to performance evidence.

## Comparison rules

XML serialization is not generally safe to compare as raw text. Each golden or
conformance case must declare whether the assertion concerns exact text, parsed
tree structure, nodes and values, messages, diagnostics, or an expected error.
Namespace nodes, prefixes, attribute order, whitespace, encodings, and output
method rules need deliberate comparison behavior.

## Conformance reporting

ADR-0006 owns the cross-suite verification-ledger invariants. Suite-specific
adapters may proceed privately, but shared record types, persistence, and report
APIs remain unstabilized until multiple assertion/environment families provide
evidence.

`scripts/inventory-xslt30-case-metadata.ps1` provides the reproducible aggregate
inventory used to plan XSLT30 preview selection. It classifies metadata shape;
it does not establish engine support or a conformance denominator by itself.

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

The suites are inputs, not executable FastXSLT tests by themselves. The private
XSLT30 slice resolves all six `template` selections and their upstream
environments, stylesheets, and XML assertions; that complete test set passes.
The overlay also conserves the complete ten-case `path` test set: `path-001`
through `path-009` pass and `path-010` remains explicitly engine-unsupported.
The paired QT3
`Axes002` named-child-axis group is inventoried but not yet executable. These
local denominators do not imply support for adjacent test sets or broad
standards conformance.
A private case-record experiment
also observes QT3 `assert-eq` and an XSLT30 compound message assertion through
suite-specific adapters. Their common projection separates selection from
execution disposition and makes unknown metadata a harness failure. A broader
harness must parse all relevant dependency and environment metadata, retain the
unaltered case identity, classify cases against ADR-0007's accepted profile,
and distinguish unsupported behavior from harness failure. Local selection and
classification belong outside the submodules.

Private ledger tests also conserve the discovered and selected denominators
across filtering, sharding, interruption, retry, and different merge orders.
These tests establish accounting invariants, not a stable report format.

The second first-party golden, `corpus/golden/template-dispatch`, exercises
exact unprefixed element-name rules through explicit `xsl:apply-templates`
selection. Duplicate patterns and modes remain unsupported so this private
case cannot be mistaken for general template-priority or built-in-rule support.
The sibling `built-in-template-rules` golden adds default child application,
built-in document/element/text behavior, and context-item value selection. It
does not broaden match priority, modes, DTD/ID behavior, or complex XPath.

Prepared-input timing remains in ignored release-mode probes. Normal
correctness gates do not enforce timing, and local ratios or phase times from a
tiny private fixture do not become cache defaults or ASP.NET performance
claims.

Allocator-request probes are also manual evidence. They use the exact-pinned,
explicitly feature-gated `allocation-counter` tool and report current-thread
requested bytes, not allocator metadata, process working set, or host memory.
The feature must remain absent from timing probes so its global allocator
wrapper cannot contaminate their results. Numeric results remain observations
rather than cross-platform assertions.

Reuse-shape probes preserve logical resource identity while comparing one
compiled stylesheet across several sources and several compiled stylesheets
across one source. Compilation and preparation remain outside timing, and
equal-byte fixtures must be identified explicitly rather than presented as
workload diversity.

Prepared-retention tests report admitted raw bytes, parser-owned capacity at
the completed-parse boundary, XDM node count, and current XDM-owned capacity as
separate classes. They must state excluded allocator, co-resident
construction-peak, index, and invocation memory rather than presenting the sum
as process memory.

Prepared-input concurrency tests distinguish concurrent immutable reads from
concurrent construction. The current explicit builder baseline permits
independent duplicate construction; cancellation and budget failures must not
publish partial entries or poison a later retry. Single-flight and waiter
behavior remain unimplemented policy, not implied guarantees.

The W3C XML Conformance Test Suite 20130923 is a reviewed but non-admitted
candidate for AR-0008. It is a dated archive rather than a Git submodule, its
root catalog uses DTD/entity composition, and its older contributor notices
require a focused rights decision before redistribution. Optional local use must
verify the recorded archive digest and classify XML edition, namespace mode,
case type, entity mode, and assertion capability before execution.

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

The `host-owned-two-stage` golden exercises that conservation rule end to end:
an earlier sealed snapshot reports the produced-but-unadmitted identity as
missing, while a new host-built snapshot can consume the same bytes only after
explicit admission.

Compiled-inspection tests assert semantic questions and bounds rather than
private debug structure. Reports must remain owned and usable after the compiled
program is dropped, inspection must not mutate compilation, and source text,
paths, matches, node identities, instruction trees, IR, and cache details stay
absent unless a later accepted contract explicitly admits them.

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
