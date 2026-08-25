# ADR-0002: Memory-Resident Execution

- Status: Accepted
- Date: 2026-08-25
- Related reviews: AR-0002, AR-0003
- Supersedes: None

## Context

FastXSLT is intended for volume use inside applications such as ASP.NET
services. Repeated file access adds latency and makes engine behavior depend on
host filesystem state. The project owner also requires isolation from Windows
Defender and other security-tool interactions that can delay, scan, or contend
with files and has identified prior Saxon file-locking experience as a failure
mode to avoid.

Keeping a source path or file handle behind a document abstraction would let
parsing, compilation, lazy resource resolution, diagnostics, generated
artifacts, or caching reopen or retain files after the caller believes loading
is complete. That would weaken snapshot determinism and complicate replacement,
deployment, cleanup, and concurrent ASP.NET operation.

## Decision

- FastXSLT compilation and transformation operate on owned or explicitly shared
  in-memory resources by default.
- Filesystem-aware host adapters may open a file or stream, copy its admitted
  bytes into a bounded resource builder, and must close the host handle before
  sealing the resource snapshot.
- A sealed snapshot never relies on the continued existence, contents, mapping,
  or open handle of an imported file.
- Core engine APIs use logical resource identity and bytes, not host paths, as
  execution inputs. A source path may survive only as diagnostic provenance and
  must never authorize reopening.
- Compilation and execution do not implicitly create intermediate artifacts,
  temporary files, spill files, memory-mapped source files, or persistent disk
  caches.
- Results are produced in memory or sent to an explicitly supplied host output
  capability. The engine does not silently choose a destination path.
- Dynamic resource access outside a sealed snapshot fails as missing or denied
  unless the caller explicitly supplies a separately authorized live resolver.
- Any disk-backed cache, spill mechanism, generated-artifact workflow, or lazy
  file resolver is an optional host mechanism requiring explicit authority,
  visible diagnostics, bounded lifecycle, and deliberate architectural review.

## Consequences

Repeated transforms can reuse stable bytes and compiled state without holding
source files open. Hosts can replace, rename, or remove imported files after the
snapshot seals, and in-flight work remains isolated from later filesystem
changes. Engine tests become deterministic because logical inputs do not mutate
through host paths.

The initial read may still be observed or delayed by host security software,
and native binaries or explicitly written results remain host files. This
decision removes engine-owned file activity after admission; it does not claim
that applications can eliminate all operating-system or security scanning.

Memory usage becomes a first-class budget. Inputs larger than admitted memory,
unbounded output, streaming, and dynamic resource sets require explicit later
design rather than transparent spill. Hosts that need persistence own when and
how in-memory results are written.

## Alternatives considered

### Keep source handles or memory maps open

This can reduce copying for some workloads but preserves file lifetime and
security-tool contention across execution, couples snapshots to mutable host
state, and complicates safe replacement.

### Reopen paths lazily

This lowers initial retention but makes behavior timing-dependent, gives paths
ambient authority, and allows one transform to observe multiple file versions.

### Transparent disk cache or spill

This supports larger workloads and persistence but recreates hidden file I/O,
locking, cleanup, security, quota, and invalidation behavior beneath an
apparently memory-resident API.

### Explicit host-owned disk mechanisms

This remains permitted as an adapter or capability because its authority and
lifecycle are visible. It is not part of the default core execution contract.

## Validation

- Import fixtures through host adapters that release all handles before sealing.
- After sealing, rename, replace, and remove original files; compilation and
  transformation must still produce the same results.
- Verify diagnostic provenance does not cause the engine to reopen a path.
- Audit core execution for implicit filesystem, temporary-file, memory-map, and
  persistent-cache dependencies.
- Exercise missing dynamic references as explicit failures without disk probes.
- Measure preload, retained/peak memory, and warm batch performance under
  caller-selected resource limits.
- Exercise snapshot replacement while old snapshots remain valid for in-flight
  ASP.NET requests.
