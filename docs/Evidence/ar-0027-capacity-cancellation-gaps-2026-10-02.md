# AR 0027 Capacity Cancellation Gaps

| Field | Value |
| --- | --- |
| Date | 2026-10-02 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Prior evidence | [Complete-source lifecycle](ar-0027-capacity-single-use-lifecycle-2026-10-02.md) |
| Scope | Test-only span-scan, reservation and freeze checkpoints |
| Disposition | Gaps measured; checkpoint design and production adoption remain open |

The shared span scan is the largest observed unchecked interval on the stress
fixture. It precedes the first XDM charge in both growth and pre-sizing, so this
is not exclusively a candidate cost. Production construction remains unchanged;
neither capacity candidate is enabled outside tests.

## Boundary controls

The [private gap tests](../../crates/fastxslt/src/runtime/prepared_input_experiment/capacity_cancellation_gap_tests.rs)
inject cancellation through test-only callbacks in the existing constructors.
Callbacks do not add polls, move charges or alter ordinary error ordering.
All six anatomy fixtures exercise these boundaries:

- Cancellation before the span scan still permits the whole scan, then returns
  XDM cancellation with zero consumed XDM-node units.
- Cancellation just before pre-sizing permits the full reservation, then is
  observed at the next node charge. Only the document node has been charged.
- Cancellation just before freeze-time shrinking permits shrinking and a
  successful constructor return. The token remains cancelled, demonstrated by
  an explicit subsequent caller check. Construction had already completed its
  semantic work; this does not establish a production cancellation defect.

A second ordinary test compares inert growth/pre-sizing observers against the
unobserved constructors for capacity, node count, string value and node charges.
The boundary controls prove where cancellation is observed, not a hard deadline,
an asynchronous allocator interruption, or a supported completion-commit rule.

## Interval measurements

Run three fresh processes without allocation instrumentation:

```text
cargo test -p fastxslt --release --features workbench measure_capacity_unchecked_intervals -- --ignored --nocapture --test-threads=1
```

Each process uses seven shapes and fixed growth/presized/frozen lane order,
with three discarded warm samples followed by sixteen measured constructions
per cell. XML parsing and source generation are outside the intervals. Each
constructed document has its expected node count checked. Callbacks and clocks
contribute overhead; fixed order, short windows and scheduling noise prohibit
competitive throughput or hard-latency claims. Process windows are 4.80-5.02 s.

The stress source contains 32,000 repeated elements with one attribute and text
each: 96,002 nodes and 96,002 XML events. It is synthetic, not a corpus case.
The following microsecond ranges are the three process medians; maxima are the
largest individual observation across those processes.

| Shape | Growth scan median | Presized scan median | Presized reserve median | Frozen shrink median |
| --- | ---: | ---: | ---: | ---: |
| wide | 4.9-14.1 | 7.1-10.5 | 2.0-2.4 | 0.4-1.3 |
| deep | 0.4 | 0.9 | 0.2-0.3 | 0.2-1.6 |
| attribute-heavy | 0.1-0.2 | 0.3 | 1.8-2.0 | 0.1-0.2 |
| text-heavy | 0.2 | 0.4-0.6 | 0.2 | 0.1-0.2 |
| namespace-heavy | 0.2-0.3 | 0.5-0.6 | 0.2-0.9 | 0.1-0.2 |
| low-repetition | 1.5-1.8 | 2.8-3.5 | 2.0-2.3 | 0.2-0.3 |
| wide-32000 | 863.6-968.7 | 581.3-769.4 | 8.1-9.8 | 26.4-30.9 |

Stress maxima are 2,314.9 us for growth scanning, 1,298.2 us for presized
scanning, 26.1 us for reservation and 79.6 us for shrinking. These are observed
maxima, not ceilings. The apparent stress scan speed difference is not an
optimization claim: the scans have different counting work and fixed lane
order, and the full preparation path is not measured by these intervals.

Growth has no explicit reservation interval; frozen construction does not
observe its shared span scan. The probe marks these as unobserved rather than
interpreting its zero placeholders as zero-cost operations. Reservation/shrink
timing does not reveal physical copied bytes: the allocator may resize in place.

## Consequences and validation

Before adopting pre-sizing, evaluate bounded cancellation polling during the
shared scan and checks around allocation. Preserve existing budget/diagnostic
ordering or explicitly review any intentional change. A post-shrink check also
needs a deliberate completion rule; do not add it merely to make a test green.
Allocator calls themselves remain non-interruptible by these engine polls.

Two ordinary tests pass. Full workspace verification passes with 1,522 core
tests and 46 ignored probes, strict Clippy, formatting, documentation, links,
conformance and unsafe-surface checks. The official WASM build passes with
filesystem hard-link fallback warnings; these timing candidates were not
measured in WASM.

The 240-line test child owns boundary faults and interval measurements. Its
233-line fixture parent only registers it; the preparation-test directory has
seven direct files. The 1,243-line XDM owner retains the constructor-local
test hooks, with no public metrics, native exports or cross-layer dependency.
No representation, scheduler, authority, ABI or corpus claim changes.
Complete transform peaks, copied-byte attribution, representative adapter
ingestion and checkpoint design remain open.

The subsequent [bounded checkpoint experiment](ar-0027-bounded-capacity-checkpoints-2026-10-02.md)
tests uncharged polling separately. It does not retroactively add checkpoints
to these interval controls or change their historical observations.
