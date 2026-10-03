# Matched fresh engine creation measurements

- Date: 2026-10-03
- Review: [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md)
- Disposition: Existing adapter baseline measured; no staging or representation selected
- Raw observations: [Three process repetitions](ar-0027-matched-engine-creation-2026-10-03.json)

Fresh full-engine creation now has matched direct Rust, native .NET and isolated
worker controls. At 500 items, creation medians are 990.10–1,043.45 us in direct
Rust, 1,053.60–1,166.50 us through native .NET and 15,129.25–16,268.10 us for a
new isolated worker. The isolated path includes process startup, so these are
not warm execution or competitive throughput numbers.

## Scope and controls

All lanes consume supplied owned source bytes and the unchanged pinned
`for-004.xsl` (377 bytes), compile and prepare a fresh engine, execute and validate
one exact result, then dispose the engine. Fixture loading/generation is outside
timing. Sources are unchanged `for03.xml` (216 bytes) and generated 5/50/500-item
documents (206/1,736/17,036 bytes), matching the existing tiered workload.
Logical identities, limits and expected results are matched. Result storage
remains caller-owned during engine disposal; this is not an allocator
release-to-zero scope.

The direct Rust lane calls the existing experimental engine constructor, not
the earlier compile-once preparation control. It includes owned byte copies.
It uses an all-feature release test binary with test instrumentation; the
adapters use production release Rust artifacts. Cross-lane subtraction is
therefore not an exact measurement of boundary overhead.

The managed probe runs without ASP.NET startup or ordinary singleton engines.
Native observation surrounds existing ABI validation, identity encoding,
creation export and outcome ownership decoding. Isolated observation surrounds
identity encoding, process-launch return, bounded initialization writes, flush
and readiness. Ordinary adapter methods remain unchanged. There are no new
exports, opcodes, acquisition capabilities, unsafe blocks or public contracts.

Each lane/fixture has eight warmups and 32 recorded samples per process.
Three fresh managed processes rotate fixture order and alternate native/isolated
lane order; three fresh Rust test processes use the same fixture rotations.
The language-process order is not randomized: managed repetitions precede Rust
repetitions. Windows are short and no convergence gate is claimed.
`PublicationEligible=false` is unconditional.

## Recorded medians

Values are microseconds. Lifecycle includes creation, the first transform with
exact validation, and engine disposal; independently calculated phase medians
need not sum to lifecycle medians.

| Order | Fixture | Rust creation | Native creation | Isolated creation | Rust lifecycle | Native lifecycle | Isolated lifecycle |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | pinned-for-004 | 33.65 | 91.60 | 13996.05 | 39.95 | 110.95 | 15738.50 |
| 0 | items-5 | 30.00 | 91.85 | 13866.10 | 36.00 | 108.95 | 15577.10 |
| 0 | items-50 | 132.85 | 161.00 | 14079.85 | 153.20 | 192.70 | 15800.30 |
| 0 | items-500 | 990.10 | 1053.60 | 15129.25 | 1136.10 | 1213.25 | 16999.80 |
| 1 | items-5 | 37.00 | 143.50 | 15207.35 | 45.05 | 171.80 | 17133.90 |
| 1 | items-50 | 155.05 | 195.25 | 14917.90 | 181.15 | 227.20 | 16696.15 |
| 1 | items-500 | 1043.20 | 1166.50 | 16268.10 | 1199.05 | 1360.10 | 18361.30 |
| 1 | pinned-for-004 | 26.95 | 137.90 | 15645.05 | 30.90 | 162.05 | 17603.75 |
| 2 | items-50 | 132.05 | 196.85 | 14022.00 | 154.15 | 233.40 | 15740.40 |
| 2 | items-500 | 1043.45 | 1082.35 | 15363.85 | 1200.50 | 1241.15 | 17265.10 |
| 2 | pinned-for-004 | 26.20 | 100.85 | 14132.00 | 30.10 | 121.20 | 15866.00 |
| 2 | items-5 | 25.20 | 94.75 | 13886.85 | 29.20 | 111.40 | 15545.20 |

The native managed-creation allocation counter records 280 bytes for generated
fixtures and 288 bytes for the pinned source. Its synchronous thread-local
scope excludes Rust allocations and the returned timing record, source/style
acquisition, transform/result allocation and disposal. No async isolated
allocation number is inferred from this counter.

Encoded isolated initialization is 681 bytes pinned and 664/2,195/17,496 bytes
at 5/50/500 items. Native supplied input-copy volume is 664 bytes pinned and
647/2,178/17,479 bytes for generated sources, including identities. These byte
counts are not retained engine bytes or process memory.

## Interpretation limits

Native ABI/identity/outcome phases are small in these windows; the combined
native creation export dominates. That export includes copying, admission,
compilation, preparation and registry publication. It cannot be called XML/XDM
preparation alone. The experiment does not separate those internal production
phases or establish a removable managed cost.

For tiny sources, isolated readiness waits have medians near 12–13.5 ms while
writes take tens of microseconds. For 500 items, writes instead take roughly
11.6–12.5 ms and readiness roughly 1.58–1.80 ms. Pipe backpressure moves startup
waiting between phases; neither phase alone identifies semantic work.
`Process.Start` returning is not worker readiness.

Native timings vary notably across fresh managed processes. Interleaving each
native creation with a process launch may affect scheduler/power/cache state;
this is a hypothesis, not measured causation. No P/Invoke speed ratio, staging
speedup or production capacity decision follows.

## Parity and cleanup

Each managed repetition checks 16 positive creations across ordinary/measured
native and isolated methods, eight failed creations over malformed XML and
production DTD denial, and 256 timed exact results. Across three recorded
processes this is 48 positive controls, 24 failure controls and 768 timed exact
results; Rust adds 384 timed exact results. Warmups are additional and uncredited.

Managed failure parity compares code, category, request identity, optional
location and detail exactly. Unclosed-element failure currently has no source
span, while DTD denial has one; the probe does not invent provenance.
Native registry ownership returns to baseline after parity and every recorded
sample. Isolated failure cleanup and teardown use the existing disposal path.
This does not measure RSS reclamation or independently prove process-memory
release.

The ordinary Rust test checks all four exact results and both default-denial
failures. Full verification passes: formatting, strict Clippy, 1,539 core tests
(49 ignored), native 18, WASM 3 and worker 18 tests, docs, links and corpus
inventory. The official WASM target check and .NET Release build pass.
The filesystem emits hard-link-cache fallback warnings; no code lint was waived.

## Reproduction and ownership review

Use the [workbench command](../../workbenches/FastXSLT.AspNet.Workbench/README.md)
with `--creation-placement-probe --creation-order=0`, then fresh processes
with orders 1 and 2. Build/copy current release Rust artifacts first.

```powershell
$env:AR0027_CREATION_ORDER = '0'
cargo test -p fastxslt --release --all-features measure_matched_full_creation -- --ignored --nocapture --test-threads=1
```

Repeat with orders 1 and 2 without simultaneous builds or validation workloads.

ADR-0004 directory pressure was reviewed: the workbench already has 25 direct
C# units, so the three related measurement units are grouped under
`CreationMeasurement`. Partial client extensions access existing adapter
primitives only; they own measurement, not engine semantics or scheduling.
The Rust measurement is a private `host_placement` child of the existing direct
lifecycle test owner. Existing constructor and worker-client behavior is not
mechanically redistributed.

Next: separate artifact/runtime conditioning from native adapter overhead,
measure reusable/fresh-source ingestion without per-document process startup,
and cover distinct/result-heavy sources, cancellation, backpressure and
co-resident storage before selecting any staging change. Layout adoption and
broader host retention remain open. OASIS exact coverage is unchanged.
