# AR 0027 Capacity Single Use Lifecycle Comparison

| Field | Value |
| --- | --- |
| Date | 2026-10-02 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Prior evidence | [Pre-sizing and semantic parity](ar-0027-node-vector-presizing-comparison-2026-10-02.md) |
| Scope | Direct Rust source admission, preparation, execution, serialization, validation and release |
| Disposition | Exact lifecycle control passes; timing benefit remains shape-dependent and non-decisional |

Pre-sizing reduces single-use wide-source lifecycle medians by 2.2-11.6% in
three runs, but other shapes and eight-reuse cells show reversals or small
differences. The earlier retained-capacity benefit remains useful evidence;
these timings do not establish a universal speedup or a reuse break-even.
Neither candidate is enabled in production.

## Implementation and scope

The [private lifecycle child](../../crates/fastxslt/src/runtime/prepared_input_experiment/capacity_lifecycle_tests.rs)
uses the same six shape fixtures as the anatomy owner and calls the existing
growth, freeze and pre-sized constructors. A stylesheet compiled once per
process returns the exact descendant-element count through `count(//*)` and
text output. Independently specified results are 1,001 wide, 256 deep, 65
attribute-heavy, 65 text-heavy, 129 namespace-heavy and 513 low-repetition.
This count workload exercises navigation but produces a tiny result; it is
not representative of result-heavy publication or the modern decimal workload.

Each sample copies the supplied source bytes into a fresh resource builder,
seals the snapshot, parses its selected member, constructs and Arc-owns XDM,
then executes and serializes one or eight independent invocations. Every
result is validated exactly. All sample-owned snapshots, documents, controls,
semantic results and serialized strings are released before the total clock
ends. Equal logical names do not imply shared prepared origins or a cache.

This is a direct Rust composition of existing private owners, not the full
supported-facade or adapter creation call. It omits prepared-set map insertion
and observation, host acquisition, managed encoding, boundary copies, worker
framing and scheduling. Compilation and fixture generation are outside timing.
It introduces no authority, mutable prepared state or alternate evaluator.

Admission timing includes byte copying and sealing. Preparation includes
control creation, XML parsing, XDM construction and Arc allocation. Execution
and serialization clocks sum the respective phases across the selected reuse
count. Total also includes validation, clock bookkeeping, invocation controls,
and owner release. Phase medians are not additive, and their differences do not
identify a particular release or allocator operation.

One and eight reuses mean one fresh source transformed one or eight times,
not one or eight independent queued documents. There is one caller thread.
No 5,000-job, concurrent-host, cold-compilation or already-warm-only comparison
was run here.

## Reproduction and controls

```text
cargo test -p fastxslt --all-features capacity_lifecycle -- --nocapture
cargo test -p fastxslt --release --features workbench measure_capacity_single_use_and_reuse_lifecycle -- --ignored --nocapture --test-threads=1
```

Three fresh release processes run without the allocation-observation feature,
after workspace gates and the WASM check. Every shape/reuse group warms all
three lanes 16 times, then samples each lane 201 times. The first lane rotates
across iterations. Shape and reuse-group order remain fixed; background load,
thermal state and steady-state convergence are not controlled. Each process
lasts 5.7-6.2 seconds, so these are exploratory windows rather than publication
quality performance estimates.

There are 32,562 exactly validated timed transform outputs per process, or
97,686 across the three runs, excluding warm-up. They are synthetic regression
observations, not additional conformance cases.

## Complete lifecycle medians

All entries are microseconds per admitted-source sample through release,
presented as growth / frozen / presized within each process. Eight-reuse values
are the whole source lifecycle with eight transforms, not per-transform latency.

| Shape | Reuses | Run 1 | Run 2 | Run 3 |
| --- | ---: | ---: | ---: | ---: |
| wide | 1 | 2066.2 / 2047.2 / 1826.8 | 1844.4 / 1899.9 / 1804.6 | 1767.5 / 1750.7 / 1662.1 |
| wide | 8 | 1983.0 / 1870.1 / 1745.8 | 1711.9 / 1700.9 / 1687.4 | 1708.8 / 1734.8 / 1751.2 |
| deep | 1 | 121.9 / 122.2 / 118.6 | 122.6 / 123.7 / 120.1 | 120.0 / 120.5 / 117.4 |
| deep | 8 | 299.7 / 298.4 / 297.0 | 301.7 / 300.9 / 295.5 | 297.6 / 296.4 / 292.9 |
| attribute-heavy | 1 | 688.1 / 700.3 / 708.9 | 550.8 / 530.6 / 526.9 | 513.8 / 517.0 / 501.9 |
| attribute-heavy | 8 | 573.0 / 568.7 / 560.3 | 760.2 / 756.0 / 742.4 | 577.0 / 595.0 / 580.2 |
| text-heavy | 1 | 170.9 / 169.6 / 169.8 | 230.7 / 226.8 / 224.2 | 205.9 / 202.5 / 199.5 |
| text-heavy | 8 | 270.3 / 264.9 / 272.7 | 290.7 / 290.0 / 287.3 | 222.4 / 224.9 / 217.0 |
| namespace-heavy | 1 | 448.0 / 437.8 / 473.4 | 504.1 / 508.7 / 492.4 | 429.5 / 431.1 / 433.5 |
| namespace-heavy | 8 | 457.9 / 451.4 / 461.6 | 621.0 / 625.0 / 611.1 | 515.8 / 510.9 / 524.1 |
| low-repetition | 1 | 867.5 / 825.3 / 813.4 | 719.9 / 766.4 / 727.6 | 681.2 / 681.6 / 672.3 |
| low-repetition | 8 | 1177.9 / 1014.0 / 1091.7 | 1231.3 / 1229.4 / 1214.8 | 1010.5 / 1027.3 / 915.8 |

Wide single-use gains repeat but range substantially across processes. Deep
single-use gains are about 2.0-2.7%. Attribute-heavy and namespace-heavy inputs
reverse direction between runs. At eight reuses, wide improves 12.0% in run 1
but regresses 2.5% in run 3. No universal latency or reuse advantage is selected.
The eight-reuse sample also follows the single-use group, so differences between
those groups must not be interpreted as a causal reuse effect alone.

For a concrete phase example, wide single-use run 1 reports growth / presized:
admission 10.9 / 14.4 us, preparation 1,756.0 / 1,556.3 us, execution 89.5 / 86.0
us, and serialization 0.5 / 0.5 us. That example localizes most measured service
time to preparation; it does not allocate the difference between phase medians
and total medians to cleanup. No standalone warm-evaluator gain is established.

## Construction release accounting

An ordinary allocation-observation test covers six shapes, three constructors
and exact/one-less node budgets: 36 construction/release combinations. Each
scope creates its own admitted bytes, snapshot, XML events, control and XDM or
failure. Successful constructions and deliberately failed constructions both
finish with zero current allocation requests and zero current requested bytes
from that scope, with nonzero total allocation requests proving the observer
was active.

This demonstrates release of measured Rust ownership in those controls, not
immediate RSS reduction, reclamation after process loss, or peak-memory parity
through complete transformation. It is separate from the timing runs and does
not change quota or accounting policy.

## Validation and remaining work

Workspace gates pass with 1,520 core tests and 45 ignored probes, including
strict Clippy, formatting, docs, links, conformance and unsafe checks. The
official WASM build passes; the candidate was not measured in WASM.

The 221-line lifecycle child owns admission-through-release measurement and
construction cleanup, depending on its 230-line fixture parent and existing
semantic owners, not sibling-test internals. Six direct files in the preparation
test directory introduce no density pressure. The parent adds only private
registration; no public facade, ABI, dependency or engine boundary changes.

Keep pre-sizing as a private memory candidate. Broader temporary-tree/path and
sealed-external-DTD parity, copied-byte and cancellation-gap attribution,
complete peak allocation through result construction, representative native/
isolated ingestion, and WASM-width measurements remain open. One/eight samples
do not determine an exact break-even reuse count. Relationship capacity remains
a separate experiment. Corpus counts and production construction are unchanged.
