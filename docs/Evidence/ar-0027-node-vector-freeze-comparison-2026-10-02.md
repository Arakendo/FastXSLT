# AR 0027 Node Vector Freeze Comparison

| Field | Value |
| --- | --- |
| Date | 2026-10-02 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Baseline | [Six-shape capacity anatomy](ar-0027-document-shape-capacity-anatomy-2026-10-02.md) |
| Candidate | Test-only node-vector `shrink_to_fit` after successful construction |
| Disposition | Measured retention benefit; not selected for production; [pre-sizing follow-up](ar-0027-node-vector-presizing-comparison-2026-10-02.md) now available |

Freeze-only shrinking reduced retained node storage in all six inputs without
changing records, relationships or accessors. Measured construction peak stayed
unchanged, but each document incurred one additional resize request and more
total requested allocation bytes. Latency varied across processes and gives no
general speedup claim. This is Candidate B, not the completed A/B comparison.

## Implementation and conservation

The test-only [XDM constructor wrapper](../../crates/fastxslt/src/xdm/owned_tree_experiment.rs)
calls the unchanged controlled constructor, then shrinks the node vector before
returning. The existing `nodes_mut` assertion requires exclusive buffer
ownership. No published prepared storage is modified; origin, node IDs, order,
provenance, names, values and relationships are retained. Relationship vectors,
namespace storage and interning are unchanged. Production callers still use the
reference constructor.

The [private comparison child](../../crates/fastxslt/src/runtime/prepared_input_experiment/capacity_comparison_tests.rs)
depends on its parent shape fixtures, not sibling internals. Three ordinary
tests verify full node/navigation/provenance parity over six shapes, equal work
charges, exact/one-less node limits, cancelled construction at four charge
positions, and exact output from unchanged W3C `for-004`/`for03.xml`. These checks
are not universal whitespace-view, typed-ID, namespace-occurrence or host parity.

Original construction charges are unchanged. The extra resize is not given a
new semantic work charge in this private prototype. Its bytes are measured by
the allocator probe; copy volume and cancellation-observation gaps during the
resize are not yet measured. Those controls must be resolved before adoption.

## Reproduction and scope

```text
cargo test -p fastxslt --all-features freeze_candidate -- --nocapture
cargo test -p fastxslt --release --all-features measure_node_freeze_allocations -- --ignored --nocapture --test-threads=1
cargo test -p fastxslt --release --features workbench measure_node_freeze_latency -- --ignored --nocapture --test-threads=1
```

Three fresh allocator runs produced identical observations. The scope includes
controlled XML parsing, XDM construction, candidate resizing and the final
document Arc; source generation/admission and transformation are outside it.
All allocations contributing to the retained document are created inside the
counter. Initial bare-document calibration was discarded; the tables use the
Arc-owned control matching the earlier preparation allocation scope.

Timing runs omit `allocation-observation`. Each of three processes warms both
lanes 32 times per fixture, then records 1,001 paired samples, alternating which
lane runs first. Compilation and transformation are not timed. Construction
includes XML parsing, XDM construction, optional resizing and Arc publication.
The second clock also includes node-count validation and document release.
Control creation, snapshot lookup, parsed-capacity observation, prepared-map
insertion/sealing, input acquisition and result handling are outside the clocks.
This is not a complete adapter-creation or transform lifecycle benchmark.

Processes ran after validation gates and the WASM check. Fixture order is fixed;
host activity and steady-state convergence are not controlled. Each timing
process took approximately 8-10 seconds including all fixtures and bookkeeping.

## Retention and allocation

Values are allocator-requested bytes. The peak is identical for both lanes in
each tested fixture; it is not a general promise about shrink operations.

| Shape | Reference retained | Frozen retained | Accounted capacity reduction | Peak in both lanes | Extra total requested bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wide | 1,183,273 | 911,961 | 22.9% | 2,358,867 | 744,496 |
| Deep | 141,844 | 78,852 | 44.4% | 346,725 | 63,984 |
| Attribute-heavy | 549,777 | 312,193 | 43.2% | 844,147 | 270,320 |
| Text-heavy | 331,793 | 300,545 | 9.4% | 398,247 | 32,240 |
| Namespace-heavy | 129,041 | 97,793 | 24.2% | 230,363 | 32,240 |
| Low-repetition | 593,973 | 467,493 | 21.3% | 1,186,395 | 381,424 |

On this allocator/build, final node-vector capacity equaled live node count.
`shrink_to_fit` does not make that a portable guarantee. Each candidate added
one allocator request; retained allocation count remained unchanged. Additional
total requested bytes equal the live node-record buffer size in these traces.
They are resize-request accounting, not measured physical copies or RSS.
Construction already reached a higher peak before freezing in these fixtures;
other shapes or vector occupancy may expose a different peak tradeoff.

## Latency observations

All values are per-process medians in microseconds. Columns show reference /
frozen within the same process; medians cannot be subtracted to infer one
particular resize or release duration.

| Shape | Run | Construction | Preparation and release |
| --- | ---: | ---: | ---: |
| Wide | 1 | 1,409.0 / 1,436.7 | 1,604.7 / 1,633.7 |
| Wide | 2 | 1,783.1 / 1,666.7 | 1,991.2 / 1,894.3 |
| Wide | 3 | 1,693.4 / 1,742.7 | 1,913.2 / 1,962.0 |
| Deep | 1 | 105.7 / 106.5 | 117.0 / 116.3 |
| Deep | 2 | 126.1 / 127.3 | 138.9 / 140.2 |
| Deep | 3 | 137.7 / 138.5 | 150.6 / 151.5 |
| Attribute-heavy | 1 | 492.4 / 495.7 | 528.9 / 531.3 |
| Attribute-heavy | 2 | 736.0 / 730.8 | 790.3 / 788.3 |
| Attribute-heavy | 3 | 755.4 / 756.7 | 808.2 / 811.9 |
| Text-heavy | 1 | 203.5 / 205.2 | 208.2 / 210.7 |
| Text-heavy | 2 | 186.4 / 186.0 | 191.1 / 190.6 |
| Text-heavy | 3 | 244.9 / 245.0 | 251.3 / 250.2 |
| Namespace-heavy | 1 | 400.7 / 399.6 | 426.6 / 424.9 |
| Namespace-heavy | 2 | 642.5 / 648.3 | 685.2 / 688.5 |
| Namespace-heavy | 3 | 646.1 / 635.2 | 684.6 / 673.9 |
| Low-repetition | 1 | 532.1 / 531.7 | 582.1 / 582.9 |
| Low-repetition | 2 | 830.9 / 830.1 | 912.5 / 911.2 |
| Low-repetition | 3 | 840.9 / 840.7 | 917.0 / 920.0 |

Retention improves consistently; timing does not establish a general speedup.
No measured warm-execution gain or reuse break-even follows: the accessors and
live records are unchanged, and no repeated-execution lane was run here.

## Next comparison and validation

The [subsequent Candidate A comparison](ar-0027-node-vector-presizing-comparison-2026-10-02.md)
measures tighter construction/pre-sizing independently. Its near-full-vector
counterexample raises freeze peak by 15.0% for only 0.63% retention savings;
the six-shape observations above remain historical and unchanged.
Counting must handle adjacent text-event coalescing and respect node-budget
admission/failure behavior rather than reserving arbitrary source-derived
capacity before controls. Do not move or batch existing semantic charges merely
to make allocation faster. Add near-full-vector occupancy cases, cancellation
gap/copy attribution, effective-view/typed-ID/namespace parity, complete
single-use transformation, and native/isolated/WASM measurements before choosing
either candidate. Relationship-vector tightening remains a separate experiment.

Full gates pass with 1,514 core tests and 42 manual probes ignored, strict
Clippy, formatting, documentation, links, conformance inventories and the unsafe
surface check. The official WASM adapter check passes. The 226-line comparison
child owns candidate conservation and measurement; its 227-line parent owns
shape fixtures and baseline anatomy. The 1,116-line XDM owner adds only the small
test-only freeze wrapper and remains cohesive. No new public API, unsafe code,
dependency, production optimization or conformance pass is admitted.
