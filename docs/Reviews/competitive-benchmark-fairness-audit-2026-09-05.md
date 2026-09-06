# Competitive Benchmark Fairness Audit

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `188d85583f3321df58ba9022c6873e6b8d633f31` |
| Review posture | Assume the harness unintentionally suppresses SaxonCS or Microsoft; look for a faster valid competitor configuration |
| Scope | ASP.NET tiered comparison, local SaxonCS adapter, Microsoft XSLT 1.0 adapter, and the relationship to AR-0021 batching |
| Status | Remediated and signed off; all findings resolved and the follow-on publication gate passed |
| Principal evidence | [FastXSLT/Saxon/Microsoft tiered rerun](../Evidence/aspnet-fastxslt-saxon-microsoft-rerun-2026-09-05.md); [AR-0021](../Architectural%20Reviews/AR-0021-bounded-isolated-transport-batching.md) |

## Resolution tracking

| Finding | Status | Resolution/evidence |
| --- | --- | --- |
| CBF-01 | Resolved | A linear `following-sibling` Microsoft challenger is roughly 37-42x faster at 500 items and about 42x lighter in managed allocation than the retained shrinking-tail oracle. |
| CBF-02 | Resolved | Seven fresh-process samples use seeded rotation, sustained time-based warm-up, distributions, confidence fields, occupancy, and an explicit publication gate. |
| CBF-03 | Resolved for exact-call family | UTF-8 `TextWriter` is the selected competitive adapter; the byte-stream implementation remains the semantic/performance oracle and focused noncompetitive fixtures retain both lanes. |
| CBF-04 | Resolved | Synchronous competitors use a plain synchronous measurement path with bounded `Parallel.For`; no `Task.FromResult` exists in their timed calls. |
| CBF-05 | Resolved | Allocation-free validation accepts only the intentional encoding-case difference and is inside both latency and throughput scopes. |
| CBF-06 | Resolved for tested envelope | Active-call instrumentation independently observed high-water 1 and 4 for every requested 1x/4x lane. |
| CBF-07 | Guardrail retained | Initialization records state that cumulative working-set observations are not comparable across engines. |
| CBF-08 | Guardrail retained | Measurement records state that managed allocation is not comparable total allocation across engines. |

Detailed implementation and measurement evidence is recorded in the
[fairness remediation tranche](../Evidence/competitive-benchmark-fairness-remediation-tranche-2026-09-05.md).

## Verdict

The harness is not fabricating its reported observations, and its evidence prose
already disclaims a general engine ranking. Compilation and source preparation
are outside the timed region for every engine; results are fully materialized
and checked; Microsoft reuses one loaded, thread-safe `XslCompiledTransform`;
Saxon reuses one immutable compiled `XsltExecutable` and creates fresh dynamic
transformers in the manner Saxonica recommends.

That is not enough to clear the benchmark for a headline competitive claim.
The hostile audit originally found:

1. **Microsoft is running a semantically adequate fixture-specific substitute
   whose shrinking-tail recursion scales differently from the operation being
   compared.** Its 500-item ratio must remain labeled as an algorithm comparison,
   not Microsoft engine performance. A less allocation-intensive XSLT 1.0
   formulation must be tested before claiming that Microsoft received its best
   reasonable lane.
2. **The competitive tier runner uses one warm-up and a fixed engine order.**
   This is weak control for managed JIT, tiered compilation, GC, cache, and
   machine-drift effects. The checked-in evidence itself reports declining and
   highly variable observations consistent with this risk.
3. **The Saxon adapter uses a byte stream, copies it with `ToArray`, and then
   decodes UTF-8 even though the official API accepts a `TextWriter`.** This is
   a plausible avoidable adapter cost and needs a controlled A/B.
4. **Both synchronous competitor APIs are forced through a per-call
   `Task.FromResult` adapter.** That makes task construction and asynchronous
   harness machinery part of their measured call even though neither API needs
   it.
5. **Microsoft alone pays output-declaration canonicalization during the
   throughput interval.** Per-call latency stops before validation, but total
   throughput and managed-allocation measurements include the case-normalizing
   `Replace`.

AR-0021 does **not** currently create an unfair Saxon/Microsoft comparison. Its
batch measurements compare FastXSLT isolated modes with the ordinary isolated
oracle; the competitor table uses one transform per call. If batch-128 is later
placed in the same headline table as one-at-a-time competitor calls, the report
must explicitly identify that as a best-practice deployment comparison and
retain a separate exact-call family.

All audit findings now have implemented remediation. The original seven-process
dataset failed the stricter publication gate because some selected post-warm-up
distributions exceeded the declared coefficient-of-variation ceiling. A
follow-on seven-process by three-round time-balanced run passed every gate. It
is eligible evidence for this declared benchmark family, not evidence that
FastXSLT is generally faster than SaxonCS or Microsoft.

## Audited code paths

- `TieredComparison.RunAsync` creates each engine/prepared input before timing,
  performs bounded time-window warm-up, then measures whole lanes in a recorded
  seeded order.
- `TieredComparison.MeasureAsync` forces managed collections before each lane,
  uses `Parallel.ForEachAsync` for genuinely asynchronous adapters, and
  validates every materialized string. Synchronous competitors use the separate
  `MeasureSync` path and bounded `Parallel.For`.
- The gitignored Saxon adapter retains one `Processor`, prepared `XdmNode`, and
  compiled `XsltExecutable`; each call creates a serializer, loads a fresh
  transformer, runs it, copies the stream, and decodes it.
- The Microsoft adapter retains one `XPathDocument` and one loaded
  `XslCompiledTransform`; each call creates a string writer and XML writer and
  materializes the output string.
- The isolated-batch benchmark rotates ordinary/batch lane order and compares
  only FastXSLT worker transport modes. It does not generate the competitive
  ratios in the tiered rerun.

## Findings

### CBF-01 — High: the Microsoft substitute requests a shrinking node-set on every recursive step

The exact XSLT 2.0 workload computes:

```xpath
sum(for $i in order-item return $i/@price * $i/@qty)
```

`XslCompiledTransform` supports XSLT 1.0 and cannot execute that expression.
The replacement stylesheet carries all remaining nodes in `$items`, reads the
first, and recursively passes `$items[position() > 1]`. For 500 nodes the
stylesheet semantically requests suffixes containing 499, 498, and so on down
to zero. An implementation that does not recognize and eliminate that pattern
may perform work proportional to the sum of those suffix lengths rather than
to 500 items.

The measured shape is consistent with that risk: Microsoft falls from 132,059
transforms/s at five items to 266/s at 500, while reported managed allocation
reaches approximately 8.79 MiB per sequential 500-item call. This is an
inference from code and scaling, not a profile proving where
`XslCompiledTransform` spends each cycle.

**Required challenge.** Add a second reviewed XSLT 1.0 formulation that carries
the current node and advances through
`following-sibling::order-item[1]`, retaining the accumulated number. Compare
both formulations for exact fixture output, initialization, allocation,
latency, and throughput. If another standards-only formulation is faster, use
the faster one in a “best reasonable Microsoft lane” and retain the current
stylesheet as an algorithmic oracle.

The Microsoft lane must continue to be described as fixture-equivalent rather
than language-equivalent. XPath 1.0 double-number behavior, XPath 2.0 typing,
and FastXSLT's admitted exact-decimal specialization are not generally the same
semantics merely because the deterministic `1.00 * 1` fixture produces the same
bytes.

**Disposition:** Resolved. The linear challenger is now the best-reasonable
Microsoft lane and the shrinking-tail implementation is retained and named as
an algorithmic oracle. The Microsoft comparison remains fixture-equivalent,
not language-equivalent.

### CBF-02 — High: one warm-up and fixed engine order do not establish managed steady state

Each tier performs one warm transform per engine. Measurements then always run
in this order:

1. FastXSLT isolated, sequential then concurrent;
2. FastXSLT native, sequential then concurrent;
3. SaxonCS, sequential then concurrent; and
4. Microsoft, sequential then concurrent.

Rust engine code is already ahead-of-time compiled. SaxonCS and
`XslCompiledTransform` execute substantial managed code; Microsoft also
documents that `XslCompiledTransform` generates IL dynamically. A single call
proves correctness but does not prove that tiered JIT/PGO, caches, and allocation
behavior have stabilized. The sequential lane always precedes the concurrent
lane, so the latter also receives more engine history.

Forced collections before each lane remove some managed heap history but do not
equalize JIT state, CPU frequency, cache state, thermal behavior, or time drift.
The competitive evidence reports that Saxon's 50-item sequential observations
ranged from about 9,560 to 39,814 transforms/s and that native results declined
across collection order. Those are large enough signals that medians alone do
not cure fixed-order bias.

**Required remediation.** Determine a warm-up duration empirically for each
engine and workload, using windows until throughput/latency stabilizes rather
than one shared magic count. Exclude all warm-up from measurement. Rotate or
randomize engine and concurrency order in every fresh process, record the seed
and position, and report per-position distributions. Prefer time-balanced
interleaving or multiple short randomized rounds over one long contiguous lane
when cross-lane drift is material.

**Disposition:** Resolved. The harness now
uses bounded time-based convergence windows and recorded seeded rotation. A
dedicated runner launches independent fresh processes, reports distributions
and convergence, and fails its publication gate unless every warm-up converges.
Extending calibration to thirty 100 ms windows produced sustained median
convergence across the seven-process audit sample. Runtime variability is
reported separately and may still fail publication eligibility.

### CBF-03 — Medium: the Saxon result adapter has an untested avoidable copy/decode path

The Saxon adapter currently performs these per request:

```text
MemoryStream -> Serializer -> MemoryStream.ToArray() -> UTF-8 decode -> String
```

`ToArray()` copies the stream's retained buffer before decoding it. Saxonica's
official .NET API also offers `Processor.NewSerializer(TextWriter)`. A
`StringWriter` whose `Encoding` property reports UTF-8, matching the existing
Microsoft adapter, can potentially produce the required managed string without
the byte-array copy and UTF-8 decoding pass.

This does not prove that the alternative is faster: Saxon's internal writer
bridge, encoding-declaration behavior, escaping, and close semantics may offset
the apparent saving. It does prove that the current lane is not yet the only
reasonable materialization pattern.

**Required challenge.** Add side-by-side byte-stream and UTF-8 `TextWriter`
Saxon adapters. Verify byte-equivalent output, including declarations,
non-ASCII text, unrepresentable characters, character references, and error
completion. Measure tiny, text-heavy, and result-heavy outputs. Retain whichever
valid adapter is faster for each declared comparison family; do not remove full
result materialization from Saxon while requiring it from FastXSLT.

**Disposition:** Resolved for the exact-call competitive family. Expected impact
on the tiny `for-004` output was material. Byte identity holds for tested UTF-8 non-ASCII and US-ASCII
character-reference output, and both destinations preserve terminating-error
completion. Exact, text-heavy, and result-heavy A/B lanes exist. A stable
universal adapter does not exist. Across seven fresh processes, `TextWriter`
was within 0.2 percent of the byte-stream median at 50 items sequentially and
produced a higher median in the other five exact-workload tier/concurrency
cells. It is selected for that UTF-8 comparison family. The byte stream remains
an oracle. Focused text/result-heavy experiments retain both lanes and make no
competitive claim.

### CBF-04 — Medium: synchronous competitors pay an artificial Task adapter

The common measurement signature is `Func<string, Task<string>>`. Saxon and
Microsoft are adapted with:

```csharp
_ => Task.FromResult(saxon.Transform())
_ => Task.FromResult(dotNet.Transform())
```

The synchronous transform completes before the returned task is awaited, but a
completed task object and the async measurement state machine remain in the
call path. This is not required by either competitor API. It also contaminates
managed-allocation figures that are later discussed per engine.

FastXSLT isolated genuinely requires asynchronous I/O. FastXSLT native is also
presented through an asynchronous pool and semaphore. A single async-shaped
harness is convenient, but convenience is not evidence that it preserves the
best synchronous throughput of the other engines.

**Required remediation.** Add a synchronous measurement path using a plain
`Func<string>` loop for Saxon and Microsoft, with a bounded `Parallel.For` or
fixed worker lanes for concurrent throughput. Measure the current async adapter
as an A/B oracle. Keep wall-clock, result materialization, validation, request
count, and maximum active transformations equivalent. Report API-call latency
separately from host-scheduling latency rather than forcing both into one value.

**Disposition:** Resolved. Synchronous competitor calls now use a plain
`Func<string>` path and bounded `Parallel.For`; the completed-task adapter was
removed from the timed competitive lanes.

### CBF-05 — Medium: output normalization charges Microsoft inside throughput but outside latency

`InvokeMeasured` stops its per-call timestamp immediately after the transform,
then calls `CanonicalizeEncoding(result)` during correctness validation. The
total throughput stopwatch includes that validation. FastXSLT and Saxon emit
`encoding="UTF-8"`; Microsoft emits `encoding="utf-8"`, so Microsoft alone
matches the replacement and creates the normalized string.

Consequences:

- latency percentiles exclude canonicalization and comparison for every lane;
- throughput includes them;
- Microsoft normally pays an additional string operation that the other lanes
  do not; and
- process-wide managed-allocation totals include that operation.

The operation is tiny relative to the current 500-item Microsoft transform but
can matter in a five-item throughput lane and makes latency, throughput, and
allocation scopes internally inconsistent.

**Required remediation.** Compare each lane against its accepted exact lexical
form, or use an allocation-free comparator that accepts only the intentional
encoding-case difference. Either include validation consistently in both
latency and throughput or exclude it from both while retaining untimed full
validation for every measured result.

**Disposition:** Resolved. The comparator is allocation-free, accepts only the
known encoding-case difference, and its work is included consistently in both
per-call latency and total throughput.

### CBF-06 — Medium: maximum concurrency is configured but not independently demonstrated

All lanes receive the same requested degree of parallelism, but they reach it
through different mechanisms: FastXSLT pools lease independent engines or
workers, while synchronous Saxon and Microsoft calls run inside
`Parallel.ForEachAsync`. A completed-task delegate may behave differently from
genuinely asynchronous pipe operations under thread-pool ramp and
scheduling. Throughput scaling does not by itself prove equal active-call
occupancy.

Microsoft's configuration is otherwise supported: Microsoft's documentation
states that a loaded `XslCompiledTransform` is thread-safe and `Transform` may
be called simultaneously from multiple threads. Saxon's immutable
`XsltExecutable` is thread-safe, and each current call owns a separate dynamic
transformer and destination.

**Required challenge.** Instrument active calls and achieved high-water
concurrency without adding timed locks to the transform path, or run fixed
long-lived lane workers released by a barrier. Report achieved occupancy,
thread-pool growth, and CPU time alongside requested concurrency. Give every
engine the same maximum active transforms, not necessarily the same scheduler
implementation.

**Disposition:** Resolved for the tested 1x/4x envelope. Every lane now reports
active-call high-water and every requested four-way measurement reached four.
Any future concurrency claim must retain and satisfy this observation.

### CBF-07 — Low: initialization and working-set rows are cumulative process snapshots

Within a tier, FastXSLT isolated and native pools are created before Microsoft
and Saxon. They remain alive while competitor initialization and execution are
measured. `ObserveHostWorkingSet()` reports the whole ASP.NET host, not bytes
owned by the newly initialized engine. Later engines therefore inherit earlier
engine allocations in their snapshot, and worker memory is reported under a
different process scope.

This does not invalidate timed warm throughput, but the initialization memory
rows cannot rank engine footprint. Run one engine per fresh process or use
owned retention instrumentation before making comparative memory claims.

**Disposition:** Resolved as a reporting guardrail. Initialization records now
state mechanically that the cumulative working-set observation is not
comparable across engines. Comparative footprint claims still require one
engine per fresh process or owned-retention instrumentation.

### CBF-08 — Low: managed allocation is not a cross-engine total-allocation metric

`GC.GetTotalAllocatedBytes` is process-wide. It counts the managed Saxon and
Microsoft engines, harness tasks, identities, validation, and ASP.NET activity,
but excludes FastXSLT's Rust allocations and isolated-worker native allocations.
The evidence discloses this important limitation. Continue to use the field for
within-lane managed regression detection, not as evidence that FastXSLT uses
less total allocation or memory than a competitor.

**Disposition:** Resolved as a reporting guardrail. Every measurement now
states mechanically that managed allocation is not comparable total allocation
across engines. Retain it only for within-lane managed regression detection.

## Claims cleared by the audit

### Compilation and source preparation are outside warm timing

All three engines compile/load the stylesheet and parse/prepare the source
before `MeasureAsync`. FastXSLT pools likewise start before timing. Initialization
is reported separately. No competitor is recompiling or reparsing the source on
each measured request.

### Saxon's fresh transformer lifecycle follows official guidance

The adapter calls `_stylesheet.Load()` per transform. This initially looks like
missed pooling, but Saxonica documents `XsltExecutable` as immutable and
thread-safe, says it is simplest to load a new `XsltTransformer` each time, and
notes elsewhere that serial transformer reuse generally offers no benefit and
can produce worse garbage-collection behavior. Transformer reuse must not be
asserted as an optimization without an A/B; the current lifecycle is idiomatic.

Relevant primary documentation:

- [SaxonCS `XsltExecutable`](https://www.saxonica.com/html/documentation12/dotnetdoc/Saxon/Api/XsltExecutable.html)
- [Saxon stylesheet caching guidance](https://www.saxonica.com/html/documentation11/using-xsl/compiling/caching.html)

### Microsoft's compiled-transform reuse and concurrency are documented

The adapter loads one `XslCompiledTransform` and reuses it. Microsoft documents
that the object is thread-safe once loaded and that `Transform` may be called
simultaneously from multiple threads. The current single compiled object is
therefore preferable to inventing four duplicate compiled instances solely to
mirror FastXSLT's pool shape.

Relevant primary documentation:

- [Microsoft `XslCompiledTransform` supplementary remarks and thread safety](https://learn.microsoft.com/en-us/dotnet/fundamentals/runtime-libraries/system-xml-xsl-xslcompiledtransform)

### Every timed call produces and checks a complete result

No competitor is timed only through evaluation while FastXSLT serializes. Each
adapter creates a destination, completes serialization, and returns a managed
string. Every result is compared against the expected result. The exact adapter
costs differ, so the selected UTF-8 exact-call destination and retained byte-
stream oracle remain separately visible. Result work has not been omitted from
a competitor lane.

### Current AR-0021 evidence does not compare batching against competitors

`IsolatedBatchComparison` measures ordinary isolated calls and batch sizes
1/8/32/128 against each other. It rotates those five FastXSLT lanes. The
FastXSLT/Saxon/Microsoft tiered comparison invokes one logical transform per
call for all engines. The present documents do not use batch-128 throughput to
calculate the published competitor ratios.

## Benchmark families required for defensible publication

### Exact-call family

Measure one logical transform per API invocation/response. Compile and prepare
outside timing, fully materialize equivalent outputs, use stabilized warm-up,
rotate engine order, and give every lane the same achieved concurrency ceiling.
This is the primary latency and closest engine-path comparison.

### Best-practice deployment family

Allow each engine its documented reasonable host pattern within an explicit
resource envelope:

- SaxonCS: one shared compiled executable, fresh transformer/destination per
  call unless an A/B proves a faster safe lane;
- Microsoft: one loaded thread-safe `XslCompiledTransform`;
- FastXSLT native: compiled/prepared engine reuse with bounded native handles;
- FastXSLT isolated: persistent workers and bounded transport batching.

Saxon and Microsoft do not need a transport-batch API merely because isolated
FastXSLT has one: they are already in-process and do not pay that pipe
transaction. The report must compare equal logical work and resource ceilings,
and label batching's latency, retention, and worker-loss tradeoffs. It must not
call this family “same invocation shape.”

### Algorithm-disclosure family

Where an engine cannot execute the exact language expression, show the closest
standards-only equivalent separately. Publish the stylesheet, complexity
concerns, and alternative formulations. Do not fold its ratio into a general
engine leaderboard.

## Required remediation sequence

1. A/B the current Microsoft shrinking-tail stylesheet against a current-node
   or other demonstrably lower-allocation XSLT 1.0 formulation.
2. Add stabilization warm-up and rotate/interleave engine and concurrency order
   across fresh processes.
3. A/B Saxon stream/UTF-8 materialization against its documented `TextWriter`
   destination.
4. Add a synchronous measurement path for the synchronous competitor APIs and
   compare it with the current completed-task adapter.
5. Remove per-engine validation asymmetry and align latency/throughput timing
   scope.
6. Demonstrate achieved concurrency and rerun enough randomized samples to
   report distributions and confidence, not only medians.
7. Preserve separate exact-call, best-practice deployment, and algorithm-
   disclosure tables.

No production engine change, public API, unsafe code, dependency admission, or
new batching contract is required to close this audit. The work belongs in the
private benchmark harness and its evidence records.

## Final disposition

All eight audit findings are resolved or retained as explicit measurement
guardrails:

- **SaxonCS:** no lifecycle sabotage was found. The exact-call family now uses
  the semantically qualified UTF-8 `TextWriter` lane and retains byte-stream
  results as an oracle; focused output-shape experiments keep both destinations.
- **Microsoft:** compilation and concurrency were sound, and the former
  shrinking-tail substitute is now an explicitly named algorithmic oracle. The
  lower-allocation linear sibling-walk formulation is the best-reasonable
  fixture-equivalent lane.
- **Batching:** no current exact-call competitive result gives FastXSLT a
  transport batch while denying a competitor the same invocation class. A
  future deployment-family table must remain separate and disclose batching's
  latency, retention, and worker-loss tradeoffs.

Closing the audit alone did not make its original numbers publication-grade.
The follow-on seven-process by three-round time-balanced run subsequently
passed the independent sample-count, destination-selection, warm-up,
measurement-duration, achieved-concurrency, and distribution-variability
gates. Any publication must retain this audit's declared benchmark-family and
non-comparability qualifications.
