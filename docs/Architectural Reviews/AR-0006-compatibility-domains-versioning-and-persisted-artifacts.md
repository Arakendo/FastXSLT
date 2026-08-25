# AR-0006: Compatibility Domains, Versioning, and Persisted Artifacts

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Standards behavior, Rust API, host ABI, diagnostics, schemas, and future durable artifacts |
| Trigger | Embedded consumers need honest compatibility boundaries before implementation creates accidental formats |
| Related ADRs | ADR-0001, ADR-0002 |
| Related evidence | AR-0001 through AR-0005 and `docs/Evidence/peer-database-documentation-review-2026-08-25.md` |

## Architectural question

Which FastXSLT compatibility domains require independent version identities and
migration policies, what pre-stability promises apply to each, and should the
project ever support persisted compiled stylesheets, caches, resource
snapshots, inspection reports, or other durable engine artifacts?

## Trigger and evidence

FastXSLT is MIT licensed for embedding. Consumers will eventually care about
Cargo/SemVer upgrades, standards behavior, diagnostic identity, managed or
native bindings, deployment artifacts, inspection schemas, and possibly
restart-time reuse of compiled stylesheets. These do not change at the same rate
and cannot safely share one ambiguous `version` field.

Tosumu separates physical storage format from consumer schema and requires
explicit migration, rejection, inspection, and downgrade behavior. FastXSLT
has no database format, but the ownership lesson applies: a standards profile
version is not a crate version; a diagnostic schema version is not an FFI ABI
version; and a compiled-artifact format is not automatically compatible because
the same project produced it.

No stable Rust API, host ABI, diagnostic catalog, inspection schema, or
serialized engine artifact exists. There is no evidence that persistence saves
time after validation, relocation, dependency checking, security, and migration
costs are included.

## Ownership and constraints

- AR-0001 owns standards editions, profiles, suites, and semantic claims.
- Cargo package policy owns Rust API compatibility and MSRV declarations.
- AR-0002 owns managed/native ABI, packaging, panic containment, and host
  lifecycle.
- AR-0004 owns diagnostic identity and serialized error compatibility.
- AR-0005 owns inspection fields and schema compatibility.
- AR-0003 owns resource snapshot identity and lifecycle; ADR-0002 forbids an
  implicit disk cache or spill path.
- Compilation owns executable validity against engine build, standards profile,
  static context, resources, extensions, and optimizer assumptions.
- The host owns publication, storage, invalidation, deployment, rollback, and
  whether a durable artifact is trusted.

Any persisted compiled form would be an explicit host-authorized feature, not a
back door around ADR-0002. It would require validation before use and must not
grant filesystem, network, extension, or host-object authority silently.

## Candidate compatibility domains

| Domain | Example identity | Primary owner |
| --- | --- | --- |
| Standards semantics | XSLT/XPath/XML/XDM profile and edition | AR-0001 and specifications |
| Rust API | Crate SemVer and MSRV | Public facade and release policy |
| Host ABI or binding | ABI or adapter contract version | AR-0002 and adapter |
| Diagnostics | Code/category contract | AR-0004 |
| Inspection | Report/schema contract | AR-0005 |
| Resource snapshot | Runtime generation, not necessarily durable | AR-0003 |
| Compiled artifact | Future format and compiler compatibility | Compilation; not admitted today |
| Serialized result | Standards serialization plus adapter envelope | Runtime and serialization |

## Alternatives

### A. Use crate SemVer for every compatibility question

Simple to communicate, but it cannot say whether standards meaning, ABI,
diagnostics, report fields, or an artifact changed. It encourages accidental
claims across unrelated domains.

### B. Introduce one global engine schema version immediately

Easy to stamp on values, but it couples independent contracts and causes either
frequent invalidation or misleading compatibility. No durable value needs it.

### C. Keep domains separate and add versions only under real pressure

Each public or durable boundary states its owner, compatibility rules, unknown
version behavior, and migration or rejection policy when it becomes real. This
avoids speculative formats but requires disciplined release documentation.

### D. Promise persisted compiled stylesheets early

This may reduce startup work, but freezes or versions IR-adjacent representation,
expands validation and untrusted-input surface, binds dependencies and
capabilities, and complicates upgrade and rollback. Measurements must justify
the cost.

### E. Never persist compiled artifacts

Always compile admitted stylesheet bytes and cache only in memory. This is
simple and evolution-friendly, but could impose unacceptable cold-start cost.
That cost is not measured.

## Findings and uncertainties

- Compatibility is a set of owned domains, not one project-wide boolean.
- FastXSLT promises no stable engine API, ABI, inspection schema, diagnostic
  catalog, or compiled format today.
- Compile-once/transform-many does not require persistence; hosts can retain
  compiled state in memory across requests.
- A durable compiled form would need provenance, content and dependency
  identity, profile and feature identity, validation, corruption handling,
  authority constraints, and explicit newer/older behavior.
- Explicit rejection and recompilation may be safer than automatic migration.
- Consumers, cold-start profiles, upgrade cadence, rollback needs, and artifact
  trust boundaries remain unknown.

## Disposition

**Incubating.** Keep pre-stability compatibility claims explicit and narrow. Do
not introduce a persisted compiled-artifact or disk-cache format without
measured consumer need and a separate accepted decision. Plans must check each
affected domain instead of asserting generic version compatibility.

## Required follow-up

- [ ] Record standards compatibility when AR-0001 reaches disposition.
- [ ] Define pre-1.0 Rust API and MSRV change policy before general publication.
- [ ] Have AR-0002 identify ABI/binding version and deployment needs.
- [ ] Have AR-0004 and AR-0005 define forward and unknown identity behavior
  before serialized contracts stabilize.
- [ ] Benchmark restart, source admission, and compilation in a real ASP.NET
  deployment before proposing persisted compiled artifacts.
- [ ] If persistence is proposed, compare recompilation, host source cache,
  portable semantic plan, and engine-private versioned artifact.
- [ ] For an admitted durable form, define identity, integrity, size bounds,
  dependency/capability binding, newer/older rejection, migration or
  recompilation, downgrade/rollback, atomic publication, and inspection.
- [ ] Propose separate ADRs for compatibility domains whose owners and evidence
  converge; do not force them into one stabilization event.

## Reopening triggers

After disposition, reopen or supersede this review when a consumer requires
rolling upgrades or rollback, cold-start compilation is a measured bottleneck,
a binding requires ABI stability, diagnostic or inspection schemas need
evolution, the standards profile changes, or persistence is proposed.

## Review history

- 2026-08-25 -- Opened as Incubating from deferred findings in the Tosumu
  documentation review.
