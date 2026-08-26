# AR-0003: Memory Resource Snapshots and Batch Transforms

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Resource loading, compilation reuse, and volume execution |
| Trigger | Volume consumers should avoid repeated file I/O and single-file call overhead |
| Related ADRs | ADR-0001, ADR-0002, ADR-0005 |
| Related evidence | Tokimu AR-0009/AR-0010 Resource Space work; `docs/Evidence/thread-pool-design-review-2026-08-25.md`; `docs/Evidence/peer-adr-0005-review-monday-2026-08-25.md`; AR-0009; AR-0010; future FastXSLT benchmarks |

## Architectural question

Should FastXSLT admit a bounded in-memory resource-set loading phase, seal it as
an immutable execution snapshot, and expose an independent unordered batch API
that reuses parsed resources and compiled stylesheets across volume work?

## Trigger and evidence

FastXSLT will be embedded in applications such as ASP.NET services where
reopening, retransferring, reparsing, and recompiling the same files per request
can dominate useful transformation work. The project owner proposes loading all
required resources into memory before executing transformations and offering an
API for a set of transformations rather than only single-file calls.

Tokimu's Resource Space review provides relevant evidence: logical store, root,
folder, and resource identity remain distinct from display names, paths, content
fingerprints, and byte retention; in-memory retention is a provider mechanism;
limits belong to caller workload policy; importers own URI interpretation; and
selected sessions must not gain ambient filesystem access. Tokimu's Weaver study
also keeps XSLT URI semantics in Weaver while a narrow adapter maps resolved
identity to selected resource bytes.

FastXSLT has not yet measured file-per-call, warmed filesystem cache, preload,
parse, compile, memory, or batch execution costs. XSLT can compute resource
references dynamically, so “load everything” is only deterministic when the
caller supplies a closed resource world or accepts explicit missing-resource
failures.

The project owner additionally requires the default execution path to avoid
retained/repeated file access because Windows Defender and other security tools
can contend with files; prior Saxon workflows are cited as negative operational
experience. ADR-0002 therefore accepts memory-resident execution and host-owned
file import as binding constraints. This review continues to own snapshot and
batch semantics, not whether core execution may silently return to disk.

## Ownership and constraints

- Hosts and adapters own file, network, database, upload, watch, and refresh
  mechanisms.
- A resource-set contract owns logical identity, qualified addresses, admitted
  bytes, metadata needed for resolution, explicit limits, and snapshot sealing.
- XSLT compilation owns include/import and static URI interpretation.
- Runtime semantics own `document()`, text/collection lookup, result-document,
  and other dynamic references admitted by the selected standards profile.
- The application owns snapshot lifetime and replacement, cache budget, batch
  policy, output publication, and whether failures abort remaining work.
- Byte sharing and fingerprints may optimize storage but never define logical
  resource identity or prove exact equality without byte comparison.

The design must preserve the SDD's static/dynamic boundary, deny ambient
authority, retain diagnostic provenance, and keep a single transform equivalent
to a batch of one.

## Candidate lifecycle

```text
ResourceSetBuilder
    admit(identity, bytes, metadata)
    enforce limits
    resolve or validate known static dependencies
            |
            v
ResourceSnapshot (sealed, immutable)
    compile reusable stylesheets
    parse/cache source documents where justified
            |
            v
TransformSet
    independent unordered requests
    isolated parameters and dynamic contexts
            |
            v
ResultSet
    result identity, value/output, messages, diagnostics, timing
```

These names communicate roles and are not accepted public Rust types.

## Alternatives

### A. File-oriented single-transform API only

Simple for occasional use, but makes repeated loading and compilation easy to
hide in the hot path and gives volume callers no shared lifecycle or result
association.

### B. Internal caches behind single-transform calls

Can accelerate common cases without a broader API, but cache identity,
invalidation, authority, memory limits, and request isolation become implicit.
Host paths risk becoming accidental resource identity.

### C. Explicit bounded resource snapshot plus batch API

Makes preload, reuse, identity, limits, and dynamic context isolation visible.
It requires additional lifecycle and diagnostic design, but keeps expensive work
outside repeated calls and is testable from ASP.NET.

### D. Always-live resolver with lazy loading

Supports dynamic documents and inputs larger than memory, but reintroduces I/O
and nondeterminism during execution. It may remain an explicit advanced
capability rather than the preferred volume path.

### E. Transformation dependency graph as the only volume API

Expresses pipelines naturally, but overcomplicates independent bulk transforms.
A batch and a graph may need separate semantics over one shared snapshot.

ADR-0005 rejects graph behavior from the initial transform-set contract. A
future graph requires a new review and cannot change existing unordered-set
semantics.

### F. Declarative transform-set builder plus bounded internal executor

The host identifies independent source, stylesheet, parameter, and logical
result relationships, then seals the set before execution. FastXSLT validates
the set and schedules requests with explicit worker/in-flight budgets. Queue and
thread mechanics remain private. This matches the selected ordering boundary,
but exact names, policies, and implementation require an executable transform
and measurements.

## Findings and uncertainties

- Compile-once/transform-many and admitted-byte reuse are strong requirements
  for volume consumers.
- A mutable builder followed by a sealed snapshot provides a clean deterministic
  boundary for in-memory work.
- Single-transform convenience should delegate to the same execution machinery
  as a batch of one.
- Independent batch execution and output-dependent pipelines are different
  concepts and should not be conflated accidentally.
- ADR-0005 now makes independent unordered execution binding: submission,
  start, and completion order are not contracts; dependent stages belong to the
  host and sibling results do not mutate the snapshot.
- A prepared-input pool may retain immutable parsed sources where measurement
  justifies it, but raw-byte, parsed-XDM, and derived-index budgets and lifetimes
  must remain distinguishable. AR-0009 now owns that retention/cache question.
- Input capacity, pending request count, worker count, and maximum in-flight
  transforms are separate policies. The discussed 5,000 inputs and 10 workers
  are benchmark parameters, not defaults.
- Memory is likely faster than repeated physical disk access, but parsing,
  compilation, OS page cache, allocation, interop, output, and peak memory may
  dominate real workloads.
- Snapshot identity, replacement generations, cache keys, concurrency,
  cancellation, output atomicity, and live-resolution policy lack evidence.
- A test-only M1 experiment now demonstrates a private mutable builder consumed
  into an immutable snapshot, explicit entry/per-entry/aggregate budgets,
  duplicate-identity rejection, equal bytes under distinct logical identities,
  and owned source/stylesheet bytes whose imported files can be renamed and
  removed immediately. It does not accept the provisional Rust names or shape.
- The first private transform compiles one stylesheet once, executes two
  identified requests over an admitted source in reverse submission order, and
  correlates equal results by request identity. Source parsing remains
  per-invocation, so this is ordering/lifecycle evidence rather than prepared
  input reuse or throughput evidence.
- Private transform-set validation now applies explicit per-set request limits,
  source denial before execution, and byte-bounded in-memory serialization.
  Exact policy ownership and defaults remain under review.
- A separate private experiment explicitly prepares selected snapshot resources
  into immutable XDM documents and functionally reuses them across the two
  volume-work shapes. It is not integrated into transform sets and supplies no
  cache, concurrency, retention, or performance contract.
- AR-0010 now owns supervision and hard-isolation guarantees. Executor mechanics
  in this review cannot assume an in-process worker can be forcibly terminated.

## Disposition

**Under Review with binding baselines from ADR-0002 and ADR-0005.** The first
vertical slice should load its source and stylesheet through a bounded builder,
release host handles, seal an immutable snapshot, and run through batch-capable
internal machinery. This does not accept public type names, unbounded
whole-workload retention, implicit caches, ordered batches, or graph execution.

## Required follow-up

- [ ] Define first-party resource identity and address fixtures without using a
  host path as identity.
- [ ] Exercise missing, denied, duplicate-name, same-content, traversal, and
  budget failures.
- [ ] Prove sealed snapshots do not change when imported files or builders
  change.
- [ ] After import, rename, replace, and remove source files on Windows and
  verify no engine handle or lazy path dependency remains.
- [x] Functionally run one stylesheet over multiple prepared sources and
  multiple stylesheets over one prepared source; performance remains unmeasured.
- [x] Resolve initial independent-batch versus output-dependent workflow
  ownership through ADR-0005; graph execution remains deferred.
- [ ] Prototype declarative transform-set sealing with duplicate request/result,
  unknown resource, and batch-budget failures before worker startup.
- [ ] Randomize scheduling and correlate results by logical identity rather than
  completion position.
- [ ] Measure raw-byte, parsed-source, derived-index, pending-request, and
  in-flight memory separately before selecting preparation or eviction policy.
- [ ] Benchmark file-per-call, warmed filesystem, preload-only, parse reuse,
  compile reuse, and full warm batch paths with peak memory reported.
- [ ] Exercise snapshot reuse and replacement through the ASP.NET workbench in
  AR-0002.
- [ ] Propose an ADR only after identity, lifetime, limit, cache, and execution
  semantics have representative evidence.

## Reopening triggers

After disposition, reopen or supersede this review when workloads exceed memory,
dynamic resource discovery is required, snapshot replacement blocks safe host
updates, cached representations exceed budgets, or a consumer needs transactional
pipeline outputs beyond the selected batch contract.

## Review history

- 2026-08-25 -- Opened as Under Review; accepted a bounded snapshot and
  batch-capable first-slice experiment without stabilizing its public API.
- 2026-08-25 -- Added test-only bounded admission and golden source/stylesheet
  handle-release evidence; public identity and lifecycle remain unresolved.
- 2026-08-25 -- ADR-0005 accepted unordered independent transform sets and
  host-owned dependent stages; prepared-input and executor policies remain under
  review.
- 2026-08-25 -- Peer review confirmed ADR-0005 without revision and moved the
  prepared-input ownership question into incubating AR-0009.
- 2026-08-25 -- The private golden set exercised reversed scheduling, stable
  result correlation, batch-of-one parity, and sibling-result invisibility.
- 2026-08-25 -- Added explicit admitted-source denial and a serialization byte
  limit without granting filesystem output authority.
- 2026-08-25 -- Added explicit selected-source preparation tied to one snapshot
  generation, with functional reuse and parse-per-invocation parity evidence.
