# ADR-0020: Official `wasm32-unknown-unknown` Build Target

- Status: Accepted
- Date: 2026-09-24
- Related review: AR-0015
- Related ADRs: ADR-0002, ADR-0003, ADR-0005, ADR-0007
- Related evidence: `docs/Evidence/wasm-browser-feasibility-and-benchmark-baseline-2026-09-24.md`
- Supersedes: None

## Context

AR-0015 asked whether FastXSLT's safe, host-neutral semantic engine could run in
a presealed WebAssembly environment without a second engine or ambient resource
authority. The first `wasm32-unknown-unknown` experiment now builds the ordinary
FastXSLT core behind a small safe adapter and executes it through a JavaScript
WASM host.

The probe did more than compile. It retained one compiled stylesheet and
prepared input across calls, executed the unchanged XSLT30 `for-004`
stylesheet, resolved a relative `xsl:include` solely from explicitly admitted
memory, preserved representative budget and invalid-input diagnostics, and
reported bounded result, retention, and linear-memory observations. Focused
native tests cover retained reuse, sealed dependency resolution, and structured
invalid-input classification. The full workspace verification gates and the
release WASM build pass.

That evidence establishes a credible build target. It does not establish a
stable JavaScript API, browser distribution, WASI/component support, broad
standards conformance, or a target-specific containment guarantee.

## Decision

FastXSLT accepts **`wasm32-unknown-unknown` as an official build target** for the
safe semantic core and its private feasibility adapter.

The official target contract is deliberately narrow:

- the `fastxslt` semantic engine used by WASM is the same engine used by direct
  Rust and the host workbenches, not a target-specific execution backend;
- the official build is represented by the unpublished
  `fastxslt-wasm-workbench` module and is compiled in continuous integration;
- first-party engine and adapter code remain safe Rust under ADR-0003;
- host values cross the adapter as owned identities, bytes, and scalar controls;
  no host borrow is retained across calls;
- resources are explicitly admitted and sealed in memory under ADR-0002;
  URL-shaped identities do not grant Fetch API, filesystem, WASI, or other
  ambient authority;
- compiled stylesheet and prepared source state may be retained within one
  module instance and reused by sequential invocations;
- results cross the boundary as owned serialized bytes and failures retain
  machine-readable diagnostic code and category; and
- one active invocation per instance is the supported reference topology for
  this target until later evidence selects another contract.

An ordinary change to the semantic core must not silently break this target. A
target incompatibility is a build regression to fix or an architectural change
to supersede deliberately, not a deferred portability concern.

### Meaning of official build support

Official support means that the repository maintains and continuously builds a
release WASM module from the shared engine, preserves focused boundary tests,
and keeps a reproducible local semantic/benchmark probe. It does not mean that
the current package is ready for application distribution.

In particular, this ADR stabilizes the **target and architectural boundary**,
not the concrete `wasm-bindgen` exports, generated JavaScript glue, module file
name, package manager metadata, or host-language types. The workbench crate
remains unpublished and its generated output remains gitignored. A later ADR
is required before a binding or packaging surface becomes public and stable.

### Dependency and tool boundary

The private adapter pins `wasm-bindgen` so its library and local CLI-generated
glue agree. The newly admitted packages declare `MIT OR Apache-2.0` licences and
are compatible with FastXSLT's MIT distribution. Dependency tooling does not
become part of engine semantics or grant authority.

The official CI build requires the Rust `wasm32-unknown-unknown` target but does
not require Node or `wasm-bindgen-cli`; those tools belong to the executable
local benchmark probe. This separates a continuously enforced target build from
runtime/tool installation policy.

## Consequences

WASM portability is now a maintained constraint on the shared engine. Platform-
specific dependencies, pointer-width conversions, synchronization assumptions,
or accidental ambient I/O that prevent the official target from compiling must
be treated as regressions.

Consumers can reasonably plan around FastXSLT producing a WASM module, but they
cannot yet depend on the experiment's JavaScript class names or generated wire
shape. Browser use still needs packaging, CSP/event-loop testing, explicit
release semantics, representative memory ceilings, and consumer-shaped API
evidence.

The target strengthens the value of the memory-resident resource model: a
sealed module can transform local host-provided data without an engine-owned
network or filesystem path. WASM itself does not prove termination,
confidentiality from the embedding host, browser-tab survival, hard memory
reclamation, or safe execution of untrusted stylesheets. FastXSLT budgets and
host/runtime containment remain separate guarantee classes.

## Non-decisions

This ADR does not:

- make the current JavaScript/Node binding public or stable;
- select browser, Node, Deno, WASI, component-model, or plugin packaging as the
  supported application profile;
- promise same-instance reentrancy, WASM threads, shared memory, or cross-
  instance compiled/prepared caches;
- authorize implicit filesystem, network, clock, randomness, or environment
  access;
- promise performance parity with native or isolated process hosts;
- claim broad XSLT, XPath, XDM, XML, or Serialization conformance;
- stabilize a cancellation, async callback, stream, or transform-set binding;
- establish a hard linear-memory or total host-process memory ceiling; or
- accept `wasm32-wasip1`, the component model, or any other WASM target.

## Alternatives considered

### Keep WASM experimental and unenforced

The target could remain a local curiosity until a browser consumer stabilizes.
The executable engine, sealed dependency, retained lifecycle, diagnostic, and
benchmark evidence now make silent regression more costly than maintaining a
small CI build. This alternative is rejected.

### Accept a stable browser binding now

The current adapter proves feasibility but has not established browser
packaging, explicit release/disposal, complete diagnostic parity, large-memory
behavior, or a consumer-owned API. This alternative is deferred.

### Accept WASI or a component interface simultaneously

Those targets have different capability and packaging models and have not been
built or exercised. Inferring them from `wasm32-unknown-unknown` would turn one
evidenced decision into several unsupported claims. This alternative is
rejected for this ADR.

## Validation

- CI installs `wasm32-unknown-unknown` and release-builds the private WASM
  adapter from the shared engine.
- Normal workspace formatting, Clippy, tests, documentation, unsafe-surface,
  Markdown-link, and corpus-source gates remain green.
- Focused adapter tests verify retained reuse after a controlled budget failure,
  sealed relative include resolution, and structured malformed-source
  classification.
- The optional local WASM workbench generates matching private bindings, runs
  the sealed-resource and diagnostic controls, executes unchanged `for-004`,
  checks exact results, and reports load, setup, warm execution, transfer,
  retained capacity, module size, and linear-memory observations.
- Future public binding work must add direct-Rust differential coverage and the
  wider parity matrix retained by the closed AR-0015 record.

## Reopening triggers

Revisit or supersede this decision if the shared engine cannot reasonably
continue compiling for `wasm32-unknown-unknown`, a consumer requires a stable
browser or language binding, a different WASM target becomes product-critical,
32-bit or linear-memory constraints require a semantic/lifecycle change, or a
host requires reentrancy, threads, live resolution, streaming results, or hard
runtime interruption guarantees.

