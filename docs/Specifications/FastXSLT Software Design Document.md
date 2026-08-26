# FastXSLT Software Design Document

| Field | Value |
| --- | --- |
| Status | Draft, pre-stability |
| Last updated | 2026-08-25 |
| Applies to | FastXSLT workspace |

## 1. Product intent

FastXSLT is a Rust-native engine for compiling and executing XSLT transforms
inside other applications. A motivating consumer is a performance-sensitive
ASP.NET application that needs to compile stylesheets once and execute them many
times. The product should be embeddable, observable, secure by explicit policy,
and capable of measuring standards conformance without substituting host-library
behavior for FastXSLT semantics.

The engine is distributed under the MIT License so consuming applications can
embed and redistribute it with minimal licensing friction. Dependency and corpus
admission must preserve that distribution model and retain all required notices.

The initial standards profile is intentionally unresolved. AR-0001 owns that
question. Until it is decided, source code and documentation must not imply
general XSLT 1.0, 2.0, or 3.0 conformance.

### Goals

- Correct, testable XSLT and XPath semantics for an explicitly selected profile.
- Reusable compiled stylesheets separated from per-transform dynamic context.
- Structured, source-located diagnostics through all engine phases.
- Explicit host resource and security policy suitable for library embedding.
- A layered verification model spanning unit, golden, conformance, differential,
  integration, and benchmark evidence.
- Performance work driven by profiles and representative transforms.
- Low-overhead host integration whose end-to-end latency and throughput can be
  measured from real consumers, including ASP.NET.

### Non-goals for the scaffold

- Selecting an XML parser or tree representation.
- Claiming a standards version or conformance level.
- Streaming, schema-aware processing, packages, or extension functions.
- A command-line interface, service, browser binding, or alternate backend.
- Stable public APIs before an end-to-end transform exercises them.

A CLI is not the primary product boundary. Host adapters are expected later,
but their concrete mechanisms are not selected by the scaffold.

## 2. Initial logical layers

FastXSLT begins as one library crate with private logical layers:

```text
Host application
      |
      v
Public facade and explicit host policies
      |
      +--------------------------+
      v                          v
Stylesheet compilation       Transformation runtime
      |                          |
      +------------+-------------+
                   v
              XSLT semantics
                   v
              XPath semantics
                   v
               XDM meaning
                   v
        Replaceable XML boundary
```

The diagram describes ownership, not one-way runtime calls in every case. XML
mechanics may be supplied by a dependency, but engine-visible node identity,
sequences, XPath, template selection, and transformation meaning belong to
FastXSLT.

Host-specific adapters, including a future ASP.NET/.NET adapter, sit above the
public facade. They translate host values, lifetime, cancellation, diagnostics,
and output without reimplementing engine semantics.

### Architectural invariants

These constraints apply even while their concrete Rust representations remain
open:

- XML dependencies provide parsing or serialization mechanics, never
  FastXSLT-visible XPath or XSLT semantics.
- A compiled stylesheet contains stylesheet-derived static state only. Source
  documents, invocation parameters, messages, clocks, resolver state, budgets,
  and other per-transform mutable state belong to a runtime invocation.
- Engine-visible nodes provide stable identity and document order for the
  lifetime required by the accepted standards profile. Neither property may be
  inferred accidentally from Rust object identity or allocation order.
- Engine layers depend on the semantic navigation and retention capabilities
  they require, not on a permanent assumption that every source is a fully
  materialized random-access tree. A concrete tree implementation remains valid
  for the first evaluator; this rule does not require speculative provider
  traits or a streaming backend.
- Resource access occurs only through explicitly supplied host capabilities.
- A sealed resource snapshot has stable identity and content for the lifetime
  of compilation and transformations that use it; host files changing beneath
  the snapshot do not mutate engine-visible inputs.
- After a host adapter admits bytes and closes its source handle, core
  compilation and execution are memory-resident by default and never reopen a
  provenance path or create implicit disk artifacts.
- Resource authority and execution budgets remain explicit even when a host
  chooses permissive policies.
- Structured diagnostics and sufficient source provenance survive parsing,
  semantic normalization, lowering, and optimization until a presentation
  boundary deliberately formats them.
- A transformation result model remains conceptually distinct from
  serialization into text, bytes, or an output sink.
- Optimization preserves every observable behavior required by the accepted
  standards profile and reference semantic tests.
- Instrumentation does not change transformation semantics or introduce ambient
  global state.

### Conceptual dependency rules

The starting dependency direction is:

```text
facade
  |---> compilation ----+
  `---> runtime --------+---> XSLT ---> XPath ---> XDM
                                                ^
XML parsing mechanics --------------------------+

diagnostics <--- emitted by every phase
host policy ---> consumed only where authority or budgets are required
```

This is an ownership rule rather than a commitment to exact Rust modules or
traits. Lower semantic layers do not invoke higher ones: XDM does not select
templates, XPath does not own stylesheet compilation, and XML parser convenience
APIs do not become query semantics. XSLT may use XPath semantics; shared helpers
must not become an unnamed alternate owner of either language. Compilation and
runtime may coordinate the layers through private representations until an
accepted ADR establishes a stable boundary.

### XML boundary

The XML layer adapts parser events or nodes and serialization mechanics. Parser
selection, DOM wrapping versus an owned arena, DTD support, and entity behavior
remain open. External entities and ambient network/filesystem access default to
unavailable until explicit policy exists.

### XDM

The XDM layer will own the engine-visible data model: node identity and order,
atomic values, sequences, names, namespaces, and conversions required by the
selected standards profile. Without knowing the physical representation, XPath
and XSLT may rely only on the semantic operations admitted by that profile,
expected to include node kind, identity, expanded name, string value,
relationships, root/document membership, and document order. Attributes,
namespaces, and typed values depend on the selected profile.

The physical representation is not yet decided. In particular:

```text
node identity  != Rust object identity
node equality  != value equality
document order != allocation order
```

### XPath

The XPath layer will own lexing, parsing, static context, dynamic context,
evaluation, type behavior, and the supported function library. It must not
delegate semantics to a host query engine whose behavior cannot be inspected or
tested against the chosen profile.

### XSLT and compilation

Compilation is conceptually a pipeline:

```text
XML stylesheet
    -> stylesheet syntax and structure
    -> static resolution and validation
    -> semantic normalization
    -> executable and optionally optimized representation
```

These stages need not become separate public types or multiple IRs. The
distinction prevents parsed syntax, semantic meaning, and a current execution
strategy from becoming architecturally identical. The reusable compiled result
must retain sufficient source provenance for diagnostics and must not capture
per-transform source documents, parameters, messages, clocks, resolver state,
budgets, or host resources as hidden global state.

Compilation may eventually attach required navigation, retention, buffering,
or evaluation capabilities to normalized expressions and templates. This is a
reserved ownership seam, not an accepted metadata schema or requirement to
implement formal streamability analysis in the first profile. One semantic
compiler should remain authoritative if later tree, streaming, or hybrid
execution strategies are admitted.

### Runtime

The runtime owns per-transformation evaluation context and result production.
Resource resolution, extension behavior, cancellation, limits, and output sinks
are host-supplied capabilities rather than ambient authority.

Result production has two conceptual phases:

```text
transformation result model -> serialization -> bytes, text, or output sink
```

The first slice may implement them together, but tests and future APIs must be
able to distinguish semantic result correctness from serialization correctness.

### Resource snapshots and volume execution

FastXSLT's preferred volume path is conceptually:

```text
host files, buffers, or application resources
                |
                v
      bounded resource loading
                |
                v
       sealed resource snapshot
          |              |
          v              v
 stylesheet compile   source parse/cache
          |              |
          +------+-------+
                 v
    independent transform requests
                 v
         result set / output sink
```

The host or an adapter owns filesystem, network, database, upload, and refresh
mechanisms. The engine owns qualified resource lookup within the admitted
snapshot and the interpretation of XSLT resource references. A display name,
host path, and content fingerprint are not resource identity.

Under ADR-0002, an adapter finishes host I/O before sealing: it copies admitted
bytes into owned/shared memory and releases source streams and file handles. A
path retained for diagnostics is inert provenance, not a lazy loading token.
Core parsing, compilation, execution, intermediate representations, and default
results do not use memory maps, temporary files, spill files, or persistent disk
caches.

Loading and mutation may occur while constructing a resource set. Compilation
and execution consume a sealed snapshot so repeated requests observe stable
bytes and deterministic lookup. The snapshot must enforce caller-selected entry,
per-entry byte, and aggregate-byte budgets. Physical byte sharing, parse caches,
and content fingerprints are provider optimizations and diagnostics; they do not
merge distinct logical resources.

A sealed snapshot owns immutable admitted resources; it does not imply that
every source is parsed into and retained as a complete tree before a batch
begins. The execution strategy may choose tree construction for the initial
engine. Any future forward-only or hybrid strategy must account for selective
materialization through explicit memory budgets and may not spill to disk
silently.

A closed snapshot cannot satisfy an unadmitted resource discovered dynamically
through `document()`, `unparsed-text()`, collections, includes/imports, or future
extensions. The engine reports an explicit missing/denied-resource condition.
An optional live resolver may be studied later, but must be an explicit
capability and cannot become ambient disk or network access.

Under ADR-0005, a transform set contains only independently executable requests.
Submission, start, execution, and completion order have no semantic meaning;
results correlate by logical request/result identity. One request cannot observe
a sibling result, and producing a result does not mutate or admit a resource.
The host sequences dependent stages and explicitly admits selected prior results
into a later snapshot. Failure collection versus fail-fast behavior,
cancellation, concurrency limits, result retention, and executor mechanics
remain open. Single-transform convenience is semantically a batch of one over
the same resource, compilation, runtime, diagnostic, and result boundaries.

### Diagnostics

Diagnostics cross every layer. They will distinguish phase, stable identity or
standards code where applicable, severity, primary source location, related
locations, and contextual details. Human formatting is an adapter over the
structured form. Lowering and optimization must retain enough provenance to
associate diagnostics with relevant stylesheet, source-document, and
host/resource locations. One event may use one as its primary location and the
others as related locations.

Boundary-facing failures should eventually preserve a stable machine identity,
a small policy category, a human-readable message, structured details, and an
underlying cause where one exists. The concrete vocabulary must be derived from
implemented failure owners rather than declared speculatively. Local modules
may retain focused error types; a single repository-wide mega-enum is not a
goal.

A reportable semantic outcome is not necessarily an operation failure. A
compilation that can reliably report unsupported syntax, or a batch that can
return independently classified request results, should preserve those
findings in its structured result where the public contract permits. Failure to
parse enough input to produce a trustworthy result, denied resource authority,
budget exhaustion, host cancellation, and internal invariant failure remain
distinguishable boundary failures. Presentation adapters must not recover this
meaning by parsing display strings.

### Observability

Observability means that a host can understand engine work without parsing
human diagnostic strings or relying on global hooks. Candidate events and
measurements include compilation and evaluation timing, resource resolution,
template invocation, expression evaluation, message/diagnostic emission, and
budget consumption.

The event vocabulary, cost model, and public interface remain open. Any design
must be opt-in or explicitly supplied, bounded where necessary, and unable to
change template selection, evaluation order where observable, results, or error
semantics.

Hosts also need read-only inspection and explainability without depending on
private engine representation. Candidate semantic snapshots include admitted
resource identities and sizes, the selected standards profile, compiled
stylesheet dependencies, static diagnostics, capability requirements, and
bounded execution summaries. Such a surface must describe supported meaning,
not stabilize a parser AST, arena index, optimizer IR, cache layout, or internal
module boundary. Exact fields and compatibility rules remain open until an
implemented slice and a real consumer exercise them.

## 3. Security and resource policy

XML and XSLT process attacker-controlled recursive structures, names, strings,
regular expressions, URIs, and imported resources. Resource exhaustion and
authority escalation are design inputs, not later hardening tasks.

Capabilities and authority determine what a transform may access:

- document, include, import, collection, and unparsed-text resolution;
- external entities and DTD processing;
- extension functions and any host objects they can access;
- diagnostic access to source text, URIs, and host paths.

Budgets and limits determine how much work it may perform:

- maximum bytes, depth, nodes, sequence growth, recursion, output, and elapsed
  work where enforceable;
- cancellation and host termination signals;
- storage retained by diagnostics, messages, tracing, and result construction.

No resolver should be interpreted as permission to use an ambient default.
Authority failures and budget exhaustion are distinct diagnostic conditions.
Whether each limit is a deterministic semantic counter, a best-effort host
safeguard, or both remains an explicit open decision.

Execution supervision cannot substitute for those local checks. An in-process
dispatcher may coordinate cooperative cancellation and bounded work but cannot
safely terminate and repair an arbitrary Rust thread. Any hard-termination
guarantee requires a discardable process or stronger isolation boundary; the
mode and evidence are tracked by AR-0010.

## 4. Public boundary

The public API remains intentionally empty during M0. The first API must be
shaped by an executable vertical slice and should make these phases visible:

1. create or supply explicit host policy;
2. compile a stylesheet and receive structured diagnostics;
3. reuse the compiled stylesheet with source input and parameters;
4. receive a semantic result plus messages or diagnostics without hidden side
   channels;
5. serialize or otherwise consume that result through an explicit boundary.

Sync versus async execution, input/output ownership, and compiled artifact
thread-safety are open decisions. Thread-safety, reentrancy, and concurrent use
must account for both compiled state and supplied host capabilities; a
thread-safe compiled representation does not make a resolver, message sink, or
output sink thread-safe automatically.

The engine boundary must be usable from Rust without an interop tax. Host
adapters may expose a narrower representation appropriate to their runtime, but
must preserve compiled-stylesheet reuse, explicit ownership, cancellation,
structured failures, and bounded resource access. ASP.NET integration must be
validated from managed caller to completed result rather than inferred from a
Rust-only benchmark.

An ASP.NET host should be able to create or replace a resource snapshot and
compiled stylesheet set outside the per-request hot path, then execute isolated
dynamic contexts concurrently when the accepted concurrency contract permits.

## 5. Verification

The normative testing tiers are defined in
[Testing Strategy](../testing-strategy.md). Passing selected corpus cases proves
only those cases under the recorded environment. Conformance reports must expose
selection and exclusion policy rather than collapsing results into an
unqualified percentage.

ADR-0006 requires suite-native case identity, an explainable disposition for
every discovered case, separate selection and execution outcomes, and conserved
denominators across filtering, sharding, interruption, retry, and merging. It
does not select the initial standards profile, a ledger schema or storage
format, or wording for a published conformance claim.

## 6. Performance

The name FastXSLT is an objective, not evidence. Performance claims require a
named workload, baseline, hardware/software environment, measurement method,
and correctness gate. Optimize measured costs only after the semantic reference
path is observable. Unsafe code remains forbidden unless a later ADR supplies
invariants, evidence, and focused verification.

ADR-0003 defines the exception policy: tests are necessary but cannot prove an
unsafe invariant. Any first-party unsafe implementation requires its own narrow
ADR, a measured need that safe Rust cannot reasonably meet, a written safety
contract, minimized and reviewable surface, safe reference behavior whenever
practical, specialized verification appropriate to the risk, and explicit
removal criteria. No unsafe implementation is currently admitted.

For embedded consumers, measurements include parsing or marshaling, interop,
stylesheet lookup/compilation policy, execution, result transfer, diagnostics,
and serialization as applicable. A faster engine core does not establish a
faster application when boundary conversion dominates the request.

Memory-first execution is expected to reduce repeated filesystem and transfer
cost for reusable workloads, but that benefit must be measured separately from
OS page caching, XML parsing, stylesheet compilation, allocation, and output
cost. Reports must include preload time, warm and cold paths, retained and peak
memory, cache hit rates, and workload reuse.

Avoiding retained and repeated file access is also an operational requirement:
hosts must be able to replace or remove imported files without engine-held
handles or later path probes. This reduces exposure to file locking and
security-tool contention after admission, while making no claim that the host's
initial read or explicit output publication escapes operating-system scanning.

## 7. Open decisions

### M0/M1 architectural decisions

- Initial XSLT/XPath standards profile and conformance suite baseline.
- XML parser and XDM physical representation.
- Intermediate representation shape and stability.
- Sync/async execution, input/output ownership, and initial public boundary.
- Transformation result model, serialization ownership, and comparison rules.
- Resource capability, cancellation, and deterministic/best-effort limit model.
- Thread-safety, reentrancy, and concurrent execution semantics for compiled
  artifacts and host capabilities.
- Initial observability events, cost constraints, and host boundary.
- Structured error/outcome categories and the read-only semantic inspection
  boundary, tracked by AR-0004 and AR-0005.
- Compatibility domains, version identifiers, and whether FastXSLT ever admits
  persisted compiled artifacts, tracked by AR-0006.
- ASP.NET/.NET integration, deployment, ownership, cancellation, and artifact
  boundary, tracked by AR-0002.
- ADR-0002 fixes logical memory-resident resource admission and sealed snapshot
  authority. ADR-0005 fixes unordered independent execution and host-owned
  workflow ordering; transformation graphs are deferred. Exact public lifecycle
  shapes remain unstabilized, while prepared-input definition, retention, and
  cache lifecycle are tracked by AR-0009.
- Execution supervision, cooperative cancellation/deadline observation, panic
  containment, worker health, and the process boundary required for hard
  termination, tracked by AR-0010.

### Deferred capability decisions

- `no_std`, WASM, and CLI requirements.
- Streaming and incremental execution, including any XSLT streaming-conformance
  claim, tracked as architectural optionality by AR-0007.
- Schema awareness and typed values beyond the initial profile.
- Packages, extension functions, and alternate execution backends.

## 8. Architecture decision policy

Decisions intentionally left open by this document must not be resolved
implicitly through implementation. Material choices affecting semantic
ownership, public boundaries, security authority, concurrency guarantees,
replaceability, or conformance claims require an Architectural Review and an
accepted ADR before they become project contracts.

A private experiment may test an alternative when its scope and reversibility
are explicit. It does not acquire architectural authority merely because it
compiles, performs well, or is convenient to reuse.
