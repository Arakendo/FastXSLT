# AR 0027 Document Shape Capacity Anatomy

| Field | Value |
| --- | --- |
| Date | 2026-10-02 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Related evidence | [Lifecycle controls](ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md), [phase attribution](ar-0027-shared-preparation-phase-attribution-2026-10-02.md) |
| Scope | Six generated source shapes, unchanged safe tree, native 64-bit release build |
| Disposition | Capacity tightening nominated for comparison; no representation or optimization accepted |

Unused node-vector capacity is a substantial and separable part of retained
storage in this matrix. It accounts for 9.4-44.4% of known document capacity,
depending on shape. Live records remain substantial too, but the text-heavy
fixture is dominated by text payload. This supports comparing capacity
construction before redesigning node records or introducing interning.

## Implementation and fixtures

The existing test-only
[capacity anatomy](../../crates/fastxslt/src/xdm/owned_tree_experiment.rs)
now separates occupied records from unused node-vector slots and reports live
child, attribute and namespace entries alongside their allocated capacities.
These are details within existing capacity terms, not extra terms added to the
total. The physical `Node`, document representation and production accounting
are unchanged. All records occupy 248 bytes on this measured native build;
relationship IDs occupy 8 bytes. These are private target-dependent layout
observations, not ABI or public type guarantees.

The [private shape matrix](../../crates/fastxslt/src/runtime/prepared_input_experiment/document_anatomy_tests.rs)
owns first-party generated XML rather than imported standards fixtures:

- **Wide:** 1,000 repeated items, each with one attribute and text.
- **Deep:** 256 nested elements and one text node.
- **Attribute-heavy:** 64 items with 16 ordinary attributes each.
- **Text-heavy:** 64 items with 4,096 text bytes each.
- **Namespace-heavy:** 128 elements with eight namespace declarations each.
- **Low-repetition:** 512 elements with distinct element/attribute names and
  distinct attribute/text values.

Every source uses the same logical identity length. Admission occurs into a
bounded sealed snapshot; controlled parsing and construction use the existing
owner, with explicit event and depth limits. The functional test checks known
node counts, string-value lengths, provenance and relationship lengths, and
reconciles the detailed capacity terms with the existing total. Namespace
declarations remain declarations: 1,024 bindings do not add eager namespace
nodes to the 130-node namespace fixture.

These synthetic shapes are an anatomy matrix, not a conformance denominator or
a representative distribution of the consumer's 5,000 files. They do not
compare XPath throughput, host adapters or alternative layouts.

## Reproduction and measurement scope

```text
cargo test -p fastxslt --all-features shape_matrix -- --nocapture
cargo test -p fastxslt --release --all-features measure_document_shape -- --ignored --nocapture --test-threads=1
```

Three fresh processes ran after workspace gates and the official WASM check.
All three reported identical allocator counts and bytes for all six inputs.
These probes measure storage, not time; the allocator feature must remain out
of separate timing binaries.

The capacity sum includes the document header, node-vector capacity, owned
relationships, strings and namespace declarations. It excludes allocator
metadata, fragmentation, Arc control blocks, snapshot storage and process
memory; these fixtures have no typed-ID indexes or effective-view overrides.
The estimate is not a universal accounting model for other retained forms.

The allocator closure covers complete controlled XML parsing and XDM
construction, retaining the resulting document beyond the closure. Source
generation and snapshot admission are outside it. Unlike the earlier baseline
anatomy probe, no parsed document is built before measurement and then moved
into the counter. Retained and peak requested bytes therefore cover allocations
created by both preparation phases on that thread. They still exclude the
allocator/OS overhead and host acquisition, source snapshot, execution and
result memory. Total requested bytes include temporary allocations and
reallocations; they are not copied-byte traffic.

## Capacity observations

All capacity values are bytes. Unused-record percentage is calculated against
the complete accounted document capacity, not just node-vector capacity.

| Shape | XML bytes | Nodes | Vector slots | Live records | Unused records | Accounted document | Unused records share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Wide | 36,007 | 3,002 | 4,096 | 744,496 | 271,312 | 1,183,201 | 22.9% |
| Deep | 1,796 | 258 | 512 | 63,984 | 62,992 | 141,772 | 44.4% |
| Attribute-heavy | 11,079 | 1,090 | 2,048 | 270,320 | 237,584 | 549,705 | 43.2% |
| Text-heavy | 262,983 | 130 | 256 | 32,240 | 31,248 | 331,721 | 9.4% |
| Namespace-heavy | 21,767 | 130 | 256 | 32,240 | 31,248 | 128,969 | 24.2% |
| Low-repetition | 20,157 | 1,538 | 2,048 | 381,424 | 126,480 | 593,901 | 21.3% |

| Shape | Child ID capacity / live | Attribute ID capacity / live | Namespace record capacity / live | Value capacity | Resource identity capacity |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wide | 40,224 / 16,008 | 32,000 / 8,000 | 0 / 0 | 15,000 | 72,048 |
| Deep | 8,224 / 2,056 | 0 / 0 | 0 / 0 | 4 | 6,192 |
| Attribute-heavy | 544 / 520 | 8,192 / 8,192 | 0 / 0 | 4,096 | 26,160 |
| Text-heavy | 2,592 / 1,032 | 0 / 0 | 0 / 0 | 262,144 | 3,120 |
| Namespace-heavy | 1,056 / 1,032 | 0 / 0 | 49,152 / 49,152 | 0 | 3,120 |
| Low-repetition | 20,512 / 8,200 | 16,384 / 4,096 | 0 / 0 | 5,120 | 36,912 |

The low-repetition fixture has 1,025 local-name occurrences and 1,025 unique
local names, plus 1,024 value occurrences and 1,024 unique values. Document-local
name/value interning would find no duplicates in those fields. Resource
identity repeats on all nodes in every fixture, but is not the dominant term.
The text-heavy fixture's 262,144 value bytes exceed its entire node-record
capacity of 63,488 bytes. Record dominance from the earlier repeated-item
baseline is therefore not universal.

## Complete preparation allocation observations

| Shape | Allocation requests | Total requested bytes | Retained requested bytes | Peak live requested bytes |
| --- | ---: | ---: | ---: | ---: |
| Wide | 19,053 | 4,300,465 | 1,183,273 | 2,358,867 |
| Deep | 1,335 | 561,674 | 141,844 | 346,725 |
| Attribute-heavy | 7,724 | 1,714,297 | 549,777 | 844,147 |
| Text-heavy | 489 | 466,041 | 331,793 | 398,247 |
| Namespace-heavy | 7,342 | 531,349 | 129,041 | 230,363 |
| Low-repetition | 9,778 | 2,170,999 | 593,973 | 1,186,395 |

Retained requested bytes are close to the known-capacity estimate for these
fixtures; that does not prove the estimate captures allocator or process
memory. Construction peak can be substantially higher than final retention.
Source bytes alone are not a general estimator of either quantity.

## Candidate and remaining work

Nominate **tighter node and relationship capacity at construction/freeze** as
the first private comparison, preserving the current record fields and
accessors. The unused-capacity arithmetic is potential, not a demonstrated
memory reduction. Pre-sizing may add counting work; post-build shrinking may
reallocate/copy and increase peak coexistence. Neither is selected here.

A candidate must preserve origin, node IDs, order, values, provenance, effective
whitespace relationships, derived namespace identity, controls and diagnostics.
Keep the unchanged tree as oracle. Measure complete preparation, construction
peak, final retention, single-use latency and reuse break-even before retaining
it. Compare node-vector and relationship changes separately for attribution.
Do not mutate published shared storage, invent an editable DOM or infer an
unsafe exception from these observations.

Field applicability by node kind, borrowed name/provenance consumers,
representative inputs, native/isolated boundary attribution, host-visible
benefit and WASM runtime anatomy remain open.

## Cohesion and validation

The private 224-line shape child owns fixture generation, capacity reconciliation
and complete-preparation allocation measurement. The preparation parent is
1,134 lines, with only module registration added in this tranche. Its named
subdirectory now contains three test children. The 1,104-line XDM owner adds
test-only observation detail in its existing capacity method, not another
representation or semantic responsibility; retain it at this checkpoint.
No direct runtime-root source file, public surface, dependency or unsafe code
is added.

All gates pass: formatting, strict all-feature Clippy, 1,511 core tests with
40 manual probes ignored, other workspace tests, documentation, links,
conformance inventories and unsafe-surface checks. The official
`wasm32-unknown-unknown` adapter check passes, but native layout numbers are not
WASM measurements. Existing build-cache warnings were not suppressed. Corpus
selection, expected outputs and compatibility counts remain unchanged.
