# Competitive Benchmark Fairness Remediation Tranche

| Field                | Value                                                                                                                                                                          |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Date                 | 2026-09-05                                                                                                                                                                     |
| Trigger              | [Competitive benchmark fairness audit](../Reviews/competitive-benchmark-fairness-audit-2026-09-05.md)                                                                          |
| Target               | .NET 10 preview workbench; local non-distributed SaxonCS-HE 13.0.0 overlay                                                                                                     |
| Main command         | `./scripts/verify-aspnet-workbench.ps1 -TargetFramework net10.0 -LocalSaxonCs -TieredBenchmark -TieredSummaryOnly -TieredRequests 250 -TieredConcurrency 4 -MeasurementRuns 3` |
| Fresh-process method | Three independent script invocations with `-MeasurementRuns 1` and distinct `-TieredOrderSeedBase` values                                                                      |
| Posture              | Fairness remediation evidence, not a general engine ranking                                                                                                                    |

## Implemented measurement corrections

- The Microsoft lane now retains the shrinking-tail stylesheet as an explicit
  algorithmic oracle and adds a second XSLT 1.0 stylesheet that advances by
  `following-sibling::order-item[1]`.
- Saxon and Microsoft use a synchronous measurement path. Their transforms no
  longer construct completed tasks merely to enter the asynchronous harness.
- Validation uses an allocation-free comparator that accepts only the known
  `UTF-8`/`utf-8` declaration spelling difference. Validation is included in
  both per-call latency and total throughput scope.
- Every asynchronous and synchronous lane records achieved active-call
  high-water. All requested four-way lanes in the three rotated samples reached
  four active transformations.
- The report records thread-pool thread count before and after each lane,
  measurement protocol, seeded lane position, and the seed itself.
- Warm-up uses bounded time-based windows and compares median throughput across
  two adjacent five-window groups, while also reporting recent median absolute
  deviation. Failure to converge remains visible.
- Initialization working-set rows and managed-allocation rows now state
  mechanically that they are not comparable total-memory measures across
  engines.

## Microsoft algorithm challenge

Both XSLT 1.0 stylesheets produced the accepted fixture result. In the three
rotated 500-item sequential samples:

| Microsoft formulation |          Throughput range | Representative managed allocation |
| --------------------- | ------------------------: | --------------------------------: |
| Shrinking-tail oracle |      243-248 transforms/s |                about 8.79 MB/call |
| Linear sibling walk   | 9,114-10,184 transforms/s |                 about 210 KB/call |

The linear formulation was roughly 37-42 times faster in these samples and
retained only about one forty-second of the oracle's managed allocation per
call. This confirms CBF-01: the previous large FastXSLT/Microsoft ratio was
dominated by the substitute algorithm and must not be presented as a Microsoft
engine-speed ratio. Future best-reasonable Microsoft tables use the linear lane;
the shrinking-tail lane remains visibly labeled as an algorithmic oracle.

The Microsoft lane remains fixture-equivalent rather than language-equivalent.
It does not execute the XPath 2.0 expression, and XPath 1.0 double arithmetic is
not generally equivalent to FastXSLT's exact-decimal activated path.

The stricter three-fresh-process replay observed 5,887-6,584 transforms/s for
the linear lane and 237-241 transforms/s for the shrinking-tail lane at 500
items sequentially. Machine/order/process variation moved the absolute linear
number, but it remained roughly 24-28 times faster while retaining the same
about-210 KB versus about-8.79 MB per-call allocation contrast.

## Warm-up and order observations

The same-process endpoint replay shuffled engine and concurrency lanes from
recorded seeds 17001, 17002, and 17003. A second replay launched three fresh
ASP.NET processes and used distinct 22101, 22201, and 22301 seeds. Achieved
position is recorded on every measurement.

The initial 16-call warm-up convergence rule was informative but not sufficient:

- all three native samples converged for every tier;
- the isolated five-item lane converged in zero of three samples within the
  192-call ceiling;
- several 500-item Microsoft and Saxon samples also reached the ceiling; and
- some lanes that met the short-window criterion still varied materially in
  later measurements.

The harness no longer hides weak warm-up or fixed order. Publication-grade
evidence uses independent fresh processes, recorded positions, distribution
statistics, confidence fields, and explicit eligibility gates. A short
convergence gate is not treated as proof of managed steady state.

The harness therefore replaced fixed 16-call windows with windows lasting at
least 100 ms (and at least eight calls), bounded at 32,768 calls. Shorter 20 ms
calibration windows still reacted excessively to managed runtime events. An initial
max/min rule proved too sensitive to ordinary GC pauses. The retained robust
rule requires median throughput drift of at most five percent between the
preceding and most recent five-window groups. Recent median absolute deviation
is recorded as a variability observation rather than conflated with central-
tendency warm-up convergence. It permits up to thirty windows after the
1.5-second calibration ceiling remained insufficient for some Saxon lanes. The sampler
records failure rather than excluding or silently extending a lane.

Early three-process calibration showed that 1.5 seconds remained insufficient
for some managed lanes. Extending the ceiling to thirty 100 ms windows produced
median-drift convergence for all selected lanes across seven fresh processes.
Median absolute deviation remains reported as runtime variability rather than a
warm-up veto. The sampler emits `PublicationEligible = false` unless there are
at least seven fresh processes, every requested concurrency is achieved, every
selected headline lane converges, a Saxon destination is explicitly selected,
and every selected distribution stays within the declared 0.20 coefficient-of-
variation ceiling. Diagnostic oracles remain reported but do not veto unrelated
headline candidates.

## Saxon destination challenge

The local Saxon adapter now exposes both:

1. `MemoryStream -> ToArray -> UTF-8 string`; and
2. Saxon's documented `TextWriter` serializer surface using a UTF-8-reporting
   `StringWriter`.

Both paths produced the accepted results for the exact, text-heavy, and
result-heavy workbench fixtures. The first exploratory focused run did not
select a universal winner:

- at 524,288 text bytes, `TextWriter` allocated about 3.17 MiB/call versus
  about 3.69 MiB/call for the byte stream, but the byte stream was slightly
  faster sequentially in that sample;
- result-heavy lanes were close and traded places by tier/concurrency; and
- concurrent text-heavy `TextWriter` observations were noisy enough to forbid
  selection from one run.

Across three fresh exact-workload processes, `TextWriter` was consistently
faster and lighter for the five-item sequential lane (36,351-38,162/s and
17,840 B/call versus 21,838-23,911/s and 25,895 B/call). That advantage did not
generalize: at 500 items the two paths overlapped and both concurrent lanes
remained highly variable. This supports an eventual declared-family choice,
not a universal adapter replacement.

An executable serializer probe also established byte identity for UTF-8
non-ASCII/astral content and US-ASCII character-reference fallback. Both paths
reported failure for a terminating `xsl:message`. The `TextWriter` path is
therefore semantically eligible for the tested serialization profiles.

Across seven fresh exact-workload processes, `TextWriter` was within 0.2 percent
of the byte-stream median at 50 items sequentially and had a higher median in
the other five tier/concurrency cells. It is therefore the selected UTF-8
exact-call competitive adapter. Serializer edge-case parity is established for
the tested profiles, and the byte-stream path remains the general oracle.
Focused text-heavy/result-heavy experiments keep both lanes because they do not
establish a universal winner and are not competitive headline families.

## Disposition

- CBF-01: resolved by the linear challenger and retained oracle.
- CBF-02: resolved by sustained warm-up, seeded fresh-process distributions,
  confidence fields, and an explicit publication gate.
- CBF-03: resolved for the exact-call competitive family by selecting UTF-8
  `TextWriter`; the stream remains an oracle and focused families retain both.
- CBF-04: resolved by dedicated synchronous measurement.
- CBF-05: resolved by allocation-free symmetric validation inside both scopes.
- CBF-06: resolved for the tested 1x/4x envelope by measured active-call
  high-water; repeat for any newly claimed concurrency.
- CBF-07 and CBF-08: retained as machine-readable non-comparability guardrails.

The audit findings were resolved before the original dataset became suitable
for publication: selected post-warm-up distributions still failed the
independent variability gate at that checkpoint.

## Follow-on time-balanced measurement control

The fixed request multiplier left some high-throughput and 500-item measurements
only tens of milliseconds long, making scheduler and collection events a large
fraction of the observed interval. The exact-call family now derives each
lane's request count from the median of its final five sequential warm-up
windows, targets approximately 500 ms after accounting for requested
concurrency, retains the caller's tier request count as a floor, and caps the
result at 500,000 calls. The actual count and elapsed time remain reported.

The publication sampler independently requires every selected measurement to
last at least 250 ms. Hitting the call cap without meeting that duration fails
closed. This is a measurement-quality control, not an engine optimization, and
the prior seven-process dataset is not retroactively treated as if it used the
new protocol. A new seven-process distribution is required before reconsidering
publication eligibility.

One 36-lane Saxon-enabled projection smoke reported no missing elapsed-time
fields and a measured-duration range of about 390-1,474 ms. A subsequent three-
fresh-process, one-round Saxon-enabled probe met every selected duration and
warm-up gate; 21 of 24 selected distributions stayed at or below 0.20 CV. The
three unstable cells were the tiny managed calls: Microsoft five-item 4x and
Saxon `TextWriter` five-item 1x/4x.

The sampler therefore supports an odd number of seeded rounds within each fresh
process, using the per-process median as one independent observation rather
than falsely counting correlated rounds as extra samples. A three-process by
three-round no-Saxon smoke produced 24 measurement groups with three independent
samples each; all warm-up, duration, occupancy, and distribution gates passed.
Its publication gate remained closed solely because three fresh processes are
below the required seven. This validates the repeated-round mechanism but does
not substitute for a seven-process run with the selected local Saxon lane.

## Full publication-gate rerun

The required follow-on used seven fresh .NET 10 processes, three seeded rounds
per process, the selected local SaxonCS `TextWriter` lane, a base request count
of 250, and requested concurrency four. Each process contributed the median of
its three rounds as one independent observation.

Every executable gate passed:

- seven independent samples were present;
- the selected Saxon destination was declared and observed;
- every selected warm-up converged;
- every requested four-way lane achieved four active calls;
- every selected measurement lasted at least 250 ms; and
- all 24 selected engine/tier/concurrency distributions stayed at or below the
  declared 0.20 coefficient-of-variation ceiling.

`PublicationEligible` was therefore `true`. The selected median throughput
observations were:

| Engine                         | 5 items 1x |  5 items 4x | 50 items 1x | 50 items 4x | 500 items 1x | 500 items 4x |
| ------------------------------ | ---------: | ----------: | ----------: | ----------: | -----------: | -----------: |
| FastXSLT isolated              |   24,903/s |    93,733/s |    21,456/s |    83,357/s |     12,589/s |     46,745/s |
| FastXSLT native                |  388,423/s | 1,158,558/s |   170,974/s |   606,447/s |     28,210/s |    107,668/s |
| Microsoft linear XSLT 1.0      |  201,358/s |   458,354/s |    61,221/s |   146,014/s |      9,098/s |     19,819/s |
| SaxonCS-HE 13.0.0 `TextWriter` |  120,060/s |   204,017/s |    44,396/s |    90,310/s |      5,797/s |     13,672/s |

These figures qualify the declared exact-call, fixture-equivalent benchmark
family under its executable gates. They do not erase the XPath/XSLT language-
surface differences, make managed/native allocation totals comparable, include
AR-0021 transport batching, or establish a general engine ranking.
