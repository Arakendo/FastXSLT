# AR-0009: Prepared Input Retention and Cache Lifecycle

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Parsed source ownership, reusable source-derived state, retention, budgets, and concurrency |
| Trigger | ADR-0005 leaves prepared-input reuse as the next volume-performance ownership question |
| Related ADRs | ADR-0001, ADR-0002, ADR-0005 |
| Related evidence | `docs/Evidence/owned-xdm-tree-experiment-2026-08-25.md`, `docs/Evidence/thread-pool-design-review-2026-08-25.md`, `docs/Evidence/peer-adr-0005-review-monday-2026-08-25.md`, and `docs/Evidence/private-prepared-input-reuse-2026-08-25.md` |

## Architectural question

What state constitutes a reusable prepared input, which owner retains it, and
how can FastXSLT bound, share, replace, and discard that state without merging
logical document identity, capturing invocation state, or turning a performance
cache into hidden semantics?

## Working terminology

A **resource** is admitted immutable bytes plus logical identity and relevant
provenance/metadata.

A **prepared input** is an immutable source-derived representation reusable by
more than one independent transformation. It may eventually include an owned
XDM document and source-only indexes. It excludes parameters, current context,
variables, messages, clocks, cancellation, result construction, resolver state,
and other invocation data.

A **retention/cache mechanism** decides whether and how long prepared state stays
available. Cache presence, a cache hit, or eviction cannot change transformation
meaning. These terms communicate roles and do not accept public Rust types.

## Trigger and evidence

The project-owner volume scenario includes thousands of in-memory documents and
bounded workers. Reusing an immutable parsed document could avoid repeating XML
parse and XDM construction for one source used by multiple stylesheets or
requests. It could also retain far more memory than the original bytes.

The first owned-XDM experiment proves a document can own its names, values,
relationships, order, and provenance after the input byte allocation is dropped.
It does not establish `Send + Sync`, representative retained size, parse cost,
stylesheet-independent indexes, replacement behavior, cache contention, or
whether reuse improves end-to-end ASP.NET work.

ADR-0005 confirms that sibling results do not mutate a work set's resource world
and that invocation state must remain isolated. ADR-0002 forbids hidden disk
spill or persistent cache fallback. AR-0007 prevents snapshot reuse from
silently making every future source an eagerly retained full tree.

## Ownership and constraints

- The host owns resource generations, snapshot replacement, total retention
  policy, and when an old generation may be released.
- A resource snapshot owns stable admitted bytes and logical identity. It does
  not automatically promise a parsed document for every entry.
- XDM owns the prepared document's node identity, document order, names, values,
  and relationships. Parser trees or lifetimes cannot become the cache contract.
- Preparation infrastructure may coordinate construction and reuse, but cache
  keys, budgets, failures, and eviction must be explicit and observable enough
  to diagnose resource pressure.
- Runtime owns per-invocation state. Executions may share prepared input only if
  no semantic or transient mutation is stored in it.
- Compiled stylesheets own stylesheet-derived static state. An index whose
  meaning depends on a stylesheet declaration, key definition, collation, or
  execution configuration is not source-only prepared state merely because it
  points at source nodes.
- Logical resource identity, base URI, snapshot generation, accepted XML/XDM
  profile, and preparation configuration may affect equivalence. A content
  fingerprint is only a storage or lookup hint and cannot merge documents.
- All retained bytes, XDM nodes, indexes, construction work, and concurrent
  waiters consume explicit budgets. No overflow spills to disk under ADR-0002.
- Eviction and construction races may affect latency and diagnostics about
  limits, but not transformation results or resource authority.

## Alternatives

### A. Parse for every invocation

This is the simplest semantic reference and minimizes retained parsed memory. It
repeats XML/XDM work and may make high-volume reuse unnecessarily expensive.

### B. Eagerly prepare every source when sealing a snapshot

Every execution receives predictable prepared state, but snapshot sealing and
peak memory scale with the entire admitted resource set even if few sources are
used. It conflicts with AR-0007's guardrail against equating sealed bytes with
eager full-tree retention.

### C. Lazily memoize prepared inputs inside a snapshot generation

The first demand constructs immutable state and later requests reuse it. The
generation supplies a strong identity/lifetime boundary. This needs single-flight
or duplicate-construction policy, failure handling, contention evidence, memory
accounting, and eviction/retry rules.

### D. Require hosts to create explicit prepared-input handles

Hosts choose exactly what to prepare and retain, making memory and lifetime
visible. It risks exposing physical XDM/cache concepts publicly and adding host
coordination before representative callers establish a useful contract.

### E. Retain prepared inputs only for one transform set or executor generation

This bounds lifetime naturally and avoids a long-lived snapshot cache, but loses
reuse across separate work sets that share a snapshot and may duplicate state
across executors.

### F. Use a process-global content-addressed cache

This maximizes opportunistic reuse but obscures authority, lifetime, budgets,
base URI, standards/configuration, tenant isolation, and document identity. It is
incompatible with the current explicit-ownership direction.

## Findings and uncertainties

- Prepared input must be immutable source-derived state; it cannot be a home for
  transformation runtime mutation.
- Raw bytes, owned XDM, source-only indexes, and stylesheet-dependent indexes are
  different retention classes with different keys and budgets.
- Snapshot generation is the strongest current candidate boundary for safe
  reuse, but eager versus lazy preparation and eviction remain unsupported by
  measurements.
- Parse-per-invocation remains the safe semantic reference behavior against
  which reuse can be tested.
- The first private transform set now uses that reference behavior for two
  requests sharing one compiled stylesheet and one admitted source. This proves
  the baseline lifecycle but contains no cache comparison or performance data.
- An implementation may physically share storage only while preserving logical
  document identity and provenance.
- Failure memoization, concurrent first access, cancellation during preparation,
  poisoning/retry, memory measurement, host visibility, and cross-snapshot reuse
  are unresolved.
- No evidence yet justifies a public pool/cache API or an ambient global cache.
- A private explicit-preparation experiment now seals selected source identities
  into immutable shared XDM documents tied to one snapshot generation. It is a
  caller-visible lifecycle experiment rather than lazy or ambient caching.
- One prepared source produces the same result as parse per invocation when
  reused by two compiled stylesheet programs. One stylesheet also executes over
  two separately prepared equal-byte resources without merging allocation or
  provenance.
- Preparation has explicit cancellation and XML/XDM work budgets. The 87-byte
  golden source produces six retained nodes and reports 1,932 bytes of owned
  representation capacity on the recorded build. That diagnostic excludes
  allocator/snapshot/temporary overhead; peak memory and timing remain
  unmeasured.
- Eight threads can concurrently read the same prepared document and compiled
  program with isolated invocation controls and equal results. This establishes
  immutable sharing for the current slice, not an executor or contention policy.
- A reproducible ignored release-mode probe now compares the complete
  parse/XDM/execute/serialize reference path with prepared lookup/execute/
  serialize while holding compilation constant. Three local runs over the
  55-byte built-in-rule golden observed 2,596.3–2,922.9 ns direct medians and
  804.2–814.9 ns prepared medians, or 3.23–3.62×. This proves measurable seam
  value only; the fixture is not consumer-representative and supplies no cache
  policy, memory, concurrency, or ASP.NET conclusion.
- Prepared-set observations now report retained raw bytes, XDM nodes, and
  XDM-owned capacity separately for each prepared identity and in aggregate.
  The 87-byte hello source reports 6 nodes and 1,932 bytes of XDM capacity; a
  generated 2,109-byte/100-item source reports 202 nodes and 63,755 bytes. The
  diagnostic excludes allocator/map/Arc overhead, construction peak, derived
  indexes, and runtime transients, so it cannot yet close the general memory
  follow-up or select budgets.

## Disposition

**Incubating.** Keep the current owned-XDM path as the safe reference. After the
first transform executes, prototype prepared-input reuse privately within one
explicit snapshot or work generation. Do not eagerly parse every admitted
resource, expose prepared-input handles, retain stylesheet-dependent runtime
indexes as source state, or introduce cross-snapshot/global caching without new
evidence.

This disposition permits an experiment; it creates no public cache, retention,
thread-safety, eviction, or performance guarantee.

## Required follow-up

- [x] Measure admitted bytes, node count, and owned-XDM payload capacity for the
  golden source under the current representation.
- [ ] Measure parse/XDM construction time, allocator-inclusive retained memory,
  and peak construction memory separately.
- [x] Run one stylesheet over multiple prepared sources and multiple stylesheets
  over one prepared source, comparing semantics with parse per invocation.
- [ ] Benchmark those workload shapes,
  comparing parse-per-invocation with reuse.
- [x] Retain a reproducible private release-mode probe that holds compilation
  constant and compares one complete direct iteration with prepared reuse.
- [x] Prove equal bytes under distinct resource identities retain distinct
  document identity and provenance when prepared.
- [x] Replace a snapshot generation while old prepared inputs remain valid only
  for explicitly retained old work.
- [ ] Test concurrent first access, duplicate construction versus single-flight,
  cancellation, construction failure, retry, and waiter behavior.
- [x] Establish that the current owned XDM representation can be shared
  immutably across concurrent readers without interior invocation mutation.
- [ ] Separate budgets and observations for raw bytes, parsed XDM, derived
  indexes, in-flight construction, and runtime transient memory.
  - [x] Observe retained raw bytes, XDM node count, and current XDM-owned
    capacity separately for explicitly prepared identities.
- [ ] Classify proposed indexes as source-only, stylesheet-derived, or
  invocation-specific before retaining them.
- [ ] Exercise eviction/reconstruction and prove it affects performance only,
  not semantic results, identity, diagnostics classification, or authority.
- [ ] Benchmark through the ASP.NET boundary before proposing defaults or a
  public lifecycle.

## Reopening triggers

Revisit or supersede this review when the first executable transform supplies
measurements, XDM sharing requires interior mutation, cache memory dominates the
workload, cross-snapshot reuse becomes necessary, a consumer needs explicit
preparation control, or another physical source strategy demonstrates a
different retention seam.

## Review history

- 2026-08-25 -- Opened as Incubating after ADR-0005 peer review identified
  prepared-input ownership as the next volume-design question.
- 2026-08-25 -- The first private transform set exercised parse-per-invocation as
  the reference path; prepared reuse remains unimplemented.
- 2026-08-25 -- Added an explicit selected-input preparation experiment tied to
  one snapshot generation. Functional reuse, reference parity, identity,
  provenance, replacement lifetime, and preparation limits pass; retention,
  concurrency, eviction, and performance policy remain Incubating.
- 2026-08-25 -- Added a release-mode direct-versus-prepared timing probe over
  the private built-in-rule golden. Retained the general benchmark and memory
  follow-ups because the tiny fixture and Rust-only boundary cannot select a
  lifecycle or establish consumer benefit.
- 2026-08-25 -- Added per-identity and aggregate raw-byte/XDM retention
  observations and exercised them on the hello and generated 100-item sources.
  Allocator-inclusive peak, indexes, and runtime transient classes remain open.
