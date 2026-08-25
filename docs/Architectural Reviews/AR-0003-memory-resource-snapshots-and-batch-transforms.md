# AR-0003: Memory Resource Snapshots and Batch Transforms

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Resource loading, compilation reuse, and volume execution |
| Trigger | Volume consumers should avoid repeated file I/O and single-file call overhead |
| Related ADRs | ADR-0001, ADR-0002 |
| Related evidence | Tokimu AR-0009/AR-0010 Resource Space work; future FastXSLT benchmarks |

## Architectural question

Should FastXSLT admit a bounded in-memory resource-set loading phase, seal it as
an immutable execution snapshot, and expose batch or transformation-graph APIs
that reuse parsed resources and compiled stylesheets across volume work?

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
    independent requests or explicit dependency graph
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

## Findings and uncertainties

- Compile-once/transform-many and admitted-byte reuse are strong requirements
  for volume consumers.
- A mutable builder followed by a sealed snapshot provides a clean deterministic
  boundary for in-memory work.
- Single-transform convenience should delegate to the same execution machinery
  as a batch of one.
- Independent batch execution and output-dependent pipelines are different
  concepts and should not be conflated accidentally.
- Memory is likely faster than repeated physical disk access, but parsing,
  compilation, OS page cache, allocation, interop, output, and peak memory may
  dominate real workloads.
- Snapshot identity, replacement generations, cache keys, concurrency,
  cancellation, output atomicity, and live-resolution policy lack evidence.

## Disposition

**Under Review with a binding memory-resident baseline from ADR-0002.** The first
vertical slice should load its source and stylesheet through a bounded builder,
release host handles, seal an immutable snapshot, and run through batch-capable
internal machinery. This does not accept public type names, unbounded
whole-workload retention, implicit caches, or a batch/graph contract.

## Required follow-up

- [ ] Define first-party resource identity and address fixtures without using a
  host path as identity.
- [ ] Exercise missing, denied, duplicate-name, same-content, traversal, and
  budget failures.
- [ ] Prove sealed snapshots do not change when imported files or builders
  change.
- [ ] After import, rename, replace, and remove source files on Windows and
  verify no engine handle or lazy path dependency remains.
- [ ] Run one stylesheet over many sources and many stylesheets over one source.
- [ ] Compare independent batch and output-dependent graph requirements.
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
