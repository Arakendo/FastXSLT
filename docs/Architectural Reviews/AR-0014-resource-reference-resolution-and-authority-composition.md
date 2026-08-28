# AR-0014: Resource Reference Resolution and Authority Composition

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-28 |
| Last reviewed | 2026-08-28 |
| Scope | Stylesheet and invocation resource references, base identity, catalogs, resolver authority, budgets, and diagnostics |
| Trigger | The first private qualified-snapshot resolver makes unresolved reference semantics and policy ownership implementation-adjacent |
| Related ADRs | ADR-0002, ADR-0005, ADR-0006 |
| Related reviews | AR-0002, AR-0004, AR-0009, AR-0010, AR-0012 |
| Related evidence | `../Evidence/private-qualified-snapshot-resolution-2026-08-28.md`; `../Evidence/private-host-owned-two-stage-workflow-2026-08-25.md` |

## Architectural question

How should FastXSLT turn stylesheet and dynamic resource references into
qualified logical identities, compose sealed-snapshot and optional live
authority, charge bounded work, and report outcomes without acquiring ambient
filesystem/network access or making host paths part of engine semantics?

## Trigger and evidence

The first private resolver now acquires the principal stylesheet by exact
qualified identity from one sealed snapshot. It charges a fixed attempt count,
checks explicit denial before snapshot membership, and distinguishes denied,
missing, invalid, unsupported, and exhausted-limit outcomes. An admitted
filename-like identity is still rejected, and a URL-shaped identity remains an
inert key rather than network authority.

That slice demonstrates a safe minimum and closes no general URI question. It
does not resolve relative references, preserve or combine base identities,
apply catalogs, load stylesheet dependencies, serve dynamic XPath/XSLT
functions, compose snapshot and live capabilities, or establish public policy
types. The XSLT30 corpus and CR-0001 both contain future dependency graphs that
will pressure these decisions, but neither currently supplies a complete
consumer-owned authority and disclosure policy.

## Ownership and constraints

- The host owns acquisition authority: which schemes, tenants, roots,
  endpoints, credentials, catalogs, and live operations are permitted.
- FastXSLT owns standards-visible reference semantics, including when a base
  identity is required, how references become logical identities, dependency
  cycle behavior, and the error identity required by the selected profile.
- A resource snapshot owns immutable admitted bytes and exact logical identity.
  Snapshot membership does not authorize an equivalent host path or URL.
- Compilation owns stylesheet-derived dependency discovery. Resolver state,
  mutable counters, clocks, cancellation, and invocation-only resources cannot
  become compiled stylesheet state.
- Invocation runtime owns dynamic resource requests and invocation-local work
  accounting. Prepared input cannot capture resolver state under AR-0009.
- ADR-0002 forbids ambient disk/network access, retained file handles, implicit
  spill, and hidden caches. Absence of a supplied resolver means absence of
  live authority.
- ADR-0005 keeps sibling results invisible until the host explicitly admits
  them into a later snapshot. Resolution cannot turn completion timing into
  implicit workflow semantics.
- Denied authority, missing resource, invalid reference, unsupported
  capability, dependency cycle, and exhausted budget remain machine-readable
  outcomes. Disclosure policy may redact detail but must not collapse the
  category a host needs to act safely.
- Resolution count, bytes admitted or returned, dependency depth, total unique
  dependencies, redirects/aliases, concurrent requests, and retained resolver
  results require explicit bounded policy where applicable.
- Observability is explicitly supplied, bounded, and semantically inert. It
  must not leak credentials or make a diagnostic callback the authority source.

## Reference identity questions

The review must separate concepts that filenames and conventional URI APIs
often conflate:

- lexical reference supplied by a stylesheet or expression;
- base logical identity and the standards rule that selects it;
- resolved logical identity used for engine identity and diagnostics;
- catalog or host mapping from one logical identity to another;
- host acquisition locator, which may contain a path, URL, database key, or
  application-specific token; and
- immutable admitted bytes and provenance retained by a snapshot.

A content hash may aid integrity or storage lookup, but cannot replace document
identity, base identity, authority, or generation ownership. URI normalization,
case handling, percent encoding, fragments, Unicode/IRI treatment, and catalog
rewrites must not be inherited accidentally from whichever helper crate is
first convenient.

## Alternatives

### Exact qualified snapshot identities only

Retain the implemented slice. Hosts pre-resolve and admit every dependency
under the exact identities FastXSLT will request. This is deterministic and
easy to bound, but pushes standards-visible base/reference behavior into each
host unless FastXSLT can expose a correct import-preparation mechanism.

### Engine-owned reference semantics over host-owned byte capabilities

FastXSLT resolves lexical references and catalog rules to logical identities,
then asks an explicit host capability for bytes. This preserves one semantic
owner across Rust, native, and isolated hosts. It requires a careful sync/async
boundary, reentrancy/concurrency rules, bounded transfer, and stable enough
request/outcome types.

### Host resolves every reference

Pass lexical and base information to the host and accept its chosen identity
and bytes. This accommodates application policy but risks semantic variation
between adapters and makes differential conformance harder unless FastXSLT
validates a strict result contract.

### Snapshot first, then optional live resolver

Consult immutable admitted state before a separately supplied live capability.
This can make common work deterministic and permit bounded misses, but lookup
order may reveal or change semantics unless identity, denial precedence,
catalogs, and generation admission are explicit. A live result must not mutate
the sealed snapshot invisibly.

### Prepare a complete dependency closure before compilation/execution

An explicit host preparation phase resolves and seals all statically known
dependencies. This fits compile-once execution and avoids callbacks in the hot
path, but dynamic functions and conditional references may still require an
invocation policy. Closure construction needs cycle, depth, count, byte, and
cancellation limits.

These alternatives may compose; this review does not select their public shape
or default precedence.

## Findings and uncertainties

- Exact qualified snapshot lookup is a useful reference path and a valid
  restricted profile, but not a general resource model.
- Authority and reference semantics are different responsibilities. The host
  decides what may be acquired; FastXSLT must not delegate standards meaning to
  each adapter.
- A URL-shaped logical identity never implies permission to access a network,
  just as a path-shaped display name never permits reopening a file.
- Denial must precede membership disclosure when policy requires that boundary.
- Compilation dependencies and invocation-time dynamic resources likely need
  different lifetimes and budgets even if they share identity machinery.
- It remains unknown whether the supported Rust facade needs synchronous,
  asynchronous, presealed-only, or multiple resolver profiles. AR-0012 and
  representative consumers must supply that pressure.
- No evidence currently selects a URI library, catalog format, callback ABI,
  resolver cache, live-result admission rule, or cross-generation sharing.

## Disposition

**Incubating.** Preserve the exact snapshot-only resolver as the private
reference and widen it only through corpus or consumer cases. Select no public
resolver trait, URI type, catalog representation, live authority, or cache.

## Required follow-up

- [x] Inventory the complete 16-case XSLT30 `decl/include` denominator and its
  34 catalog-declared secondary stylesheet references. Nominate `include-0401`
  as the first bounded dependency case while retaining all cases in an explicit
  first-party classification overlay.
- [ ] Record CR-0001's complete immutable Web3D dependency graph, base-URI and
  catalog expectations, parameters, and host authority before using it to shape
  a supported resolver boundary.
- [ ] Test relative-reference and base-identity semantics independently from
  host acquisition, using sealed in-memory resources only.
- [ ] Establish dependency count/depth/byte/cycle accounting and prove failed
  resolution cannot partially mutate compiled or snapshot generations.
- [ ] Exercise denied versus missing disclosure through the eventual Rust and
  selected .NET host profiles without parsing display strings.
- [ ] Compare presealed closure, callback/live, and hybrid lifecycle costs on a
  representative multi-resource workload before selecting supported profiles.
- [ ] Review URI parsing/normalization and catalog dependencies for standards
  behavior, portability, maintenance, and license before admission.

## Reopening triggers

- An admitted XSLT30 case requires `xsl:include`, `xsl:import`, `document()`,
  `doc()`, `collection()`, or `unparsed-text()`.
- A consumer supplies a multi-resource transform with authoritative base,
  catalog, trust, and deployment requirements.
- Host integration requires asynchronous acquisition, credentials, tenant
  isolation, or hard-isolated resolver transport.
- Resolution or dependency preparation becomes a measured latency, retention,
  contention, or denial-of-service pressure.

## Review history

- 2026-08-28 -- Opened as Incubating after the private exact qualified-snapshot
  resolver made the remaining authority and reference-semantics choices
  implementation-adjacent.
