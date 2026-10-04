# Persistent worker ingestion control

- Date: 2026-10-03
- Review: [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md)
- Baseline: [Fresh engine creation](ar-0027-matched-engine-creation-2026-10-03.md)
- Observations: [Three 5000 job repetitions](ar-0027-persistent-worker-ingestion-2026-10-03.json)
- Disposition: Startup removed from ingestion timing; no staging optimization selected

A private adapter probe now feeds a sequential 5,000-job mixed queue through
one persistent isolated worker and sequential native engine replacement.
All 30,000 timed outputs match across three fresh managed processes.
At 500 items, persistent isolated initialization medians are 816.15–896.90 us,
versus 15,129.25–16,268.10 us with a newly launched worker in the earlier
experiment. These separately conditioned windows identify startup scope,
not an exact overhead subtraction or a new preparation speedup.

## Scope

Each queue cycles through pinned `for03.xml` and generated 5/50/500-item sources
using the unchanged 377-byte `for-004.xsl`. Source acquisition/generation,
identity assembly, seed-engine creation, process startup and final shutdown
are outside job timing. Every job has a distinct logical source identity.
Equal bytes do not mean the document is reused. The queue has 5,000 jobs per
lane, one job active per lane, and 1,250 recorded observations per fixture;
32 extra warmups precede measurement.

Native creates a replacement through the existing export, retains the prior
engine until creation succeeds, releases it and transforms the new document.
Isolated sends the existing initialization opcode to the same process, waits
for readiness and transforms. Existing worker semantics build a replacement
and publish only on success. Both lanes recompile and prepare every source.
Neither is compile-once ingestion, concurrent generation publication, or a
supported replacement API. No worker protocol, Rust code or unsafe surface
changes.

The initialization phase includes old-engine retirement in isolated mode.
Native measures old-engine retirement separately. The circular order is
pinned → 5 → 50 → 500 → pinned; rotating its starting point does not change
those predecessors. Consequently the pinned job retires a 500-item tree.
Its native retirement median is 52.3–53.5 us, much larger than its own tiny
prepared tree would suggest. The group label describes the incoming source,
not the retired generation.

Native/isolated lane order alternates within jobs. Three fresh .NET 10.0.12
processes use starting offsets 0/1/2. These instrumented windows have no
publication-convergence gate and always report `PublicationEligible=false`.
One mixed sequential stream is not the production eight-worker pool and these
phase medians are not queue-throughput estimates.

## Recorded medians

Values are microseconds. Through-result includes initialization/replacement,
old-engine retirement and one exact transform/validation. The current engine
remains retained for the next replacement; final teardown is outside this clock.

| Order | Incoming fixture | Native creation | Native old retirement | Native through result | Isolated initialization | Isolated through result |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | pinned | 46.30 | 52.30 | 105.75 | 208.55 | 283.80 |
| 0 | items-5 | 58.85 | 3.70 | 71.50 | 116.60 | 179.05 |
| 0 | items-50 | 99.55 | 1.90 | 110.00 | 206.45 | 282.65 |
| 0 | items-500 | 639.35 | 8.10 | 692.15 | 896.90 | 1023.75 |
| 1 | pinned | 45.55 | 52.30 | 105.25 | 204.70 | 280.60 |
| 1 | items-5 | 58.30 | 3.70 | 71.00 | 114.80 | 176.65 |
| 1 | items-50 | 100.45 | 1.90 | 110.85 | 197.45 | 270.15 |
| 1 | items-500 | 654.75 | 7.70 | 706.50 | 850.10 | 976.55 |
| 2 | pinned | 46.50 | 53.50 | 107.00 | 202.75 | 277.40 |
| 2 | items-5 | 58.80 | 3.80 | 71.90 | 115.80 | 176.55 |
| 2 | items-50 | 104.10 | 1.90 | 114.90 | 195.65 | 268.35 |
| 2 | items-500 | 688.95 | 8.10 | 740.25 | 816.15 | 943.20 |

Larger isolated writes no longer absorb a process-start wait: 500-item write
medians are 42.30–43.15 us, compared with roughly 11.6–12.5 ms in the fresh-worker
probe. Readiness still combines decoding, admission, compilation, preparation
and retiring the prior engine. Native creation is also a combined boundary
operation. No removable managed loop or copy has been isolated.

## Failure and cleanup controls

Before each measured queue, malformed XML and default DTD denial are compared
across native and persistent isolated creation for exact code, category,
request, optional location and detail. Both previously admitted engines then
still return the pinned exact result. An oversized source is rejected before
any opcode is written; the same isolated worker remains usable.

A separate worker is killed before reinitialization. The adapter rejects the
lost transaction, retires it without implicit retry, and an explicitly created
fresh worker returns the exact result. Unknown/partial framing failures retire
the process rather than sending graceful shutdown into a desynchronized pipe.
The retirement path releases its operation gate before disposal, which itself
acquires that gate.

Each repetition conserves five same-generation recovery results and one fresh
worker recovery result, beyond warmups and the 10,000 timed outputs.
The main worker PID remains unchanged throughout the queue. Native registry
ownership returns to its initial baseline after final disposal.

These checks are not full cancellation/backpressure/partial-transfer coverage.
There are no retained-capacity or allocator/RSS measurements here. Original
fixture arrays, report samples and identities have different host lifetimes;
two engine generations can briefly coexist during successful replacement.
No hard memory bound is inferred from the sequential job count.

## Reproduction and next checkpoint

After the [existing release build/copy steps](../../workbenches/FastXSLT.AspNet.Workbench/README.md):

```powershell
dotnet workbenches/FastXSLT.AspNet.Workbench/bin/Release/net10.0/FastXSLT.AspNet.Workbench.dll --persistent-ingestion-probe --ingestion-jobs=5000 --creation-order=0
```

Repeat in fresh processes with orders 1 and 2. Default job count 128 is a smoke;
the private CLI accepts 128–5,000. All acquisition stays host-owned.

The new units remain in the existing private `CreationMeasurement` subject,
which now has five files. Client extensions own framing observation and
sequential experiment composition, not semantics, scheduler policy or ABI.
No existing client implementation is redistributed. Production constructors,
capacity policy and OASIS credit remain unchanged.

The .NET Release build passes with zero warnings/errors. Full Rust gates,
documentation/conformance checks and the official WASM target are rerun as
conservation gates despite no Rust change.

Next: matched compile-once fresh-source ingestion requires an explicit shared
lifecycle seam rather than pretending repeated initialization reuses compilation.
Separately measure distinct/result-heavy source payloads, cancellation, pipe
backpressure and co-resident state before choosing a staging relocation.
