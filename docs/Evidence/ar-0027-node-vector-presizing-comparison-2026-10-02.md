# AR 0027 Node Vector Presizing Comparison

| Field | Value |
| --- | --- |
| Date | 2026-10-02 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Reference | [Document anatomy](ar-0027-document-shape-capacity-anatomy-2026-10-02.md) and [freeze comparison](ar-0027-node-vector-freeze-comparison-2026-10-02.md) |
| Candidate | Test-only node-vector pre-sizing during existing construction |
| Disposition | Promising allocation result; no production selection |

Pre-sizing reaches the same retained capacity as freeze-time shrinking on the
six original shapes, while reducing allocator-requested peak by 15.8-48.4%.
It avoids repeated node-buffer growth without changing records, relationships
or accessors. A seventh, nearly full vector exposes a freeze counterexample:
shrinking saves only 0.63% of known document capacity but increases peak by
15.0%. These are private preparation measurements, not host performance or
process-memory guarantees.

## Candidate and conservation

The [XDM owner](../../crates/fastxslt/src/xdm/owned_tree_experiment.rs) retains one
controlled constructor. The candidate counts required slots during its existing
event-span scan, including attributes and adjacent text-event coalescing. It
does not add a second traversal. Checked count overflow falls back to ordinary
growth. After the original document-node charge, reservation is eligible only
when the remaining node budget covers the planned nodes. A budget that cannot
cover the complete document takes ordinary growth and fails at the original
per-node charge point, rather than admitting an oversized candidate buffer.

Reservation allocates uninitialized capacity, not semantic nodes. Every node
still receives the original charge before materialization; charges are neither
batched nor moved. Counting arithmetic is additional test-only work within an
existing bounded traversal, without a new semantic charge. Cancellation during
that scan or allocation is not newly observed. Measuring and bounding those
observation gaps remains an adoption prerequisite.

The [comparison child](../../crates/fastxslt/src/runtime/prepared_input_experiment/capacity_comparison_tests.rs)
checks seven shapes and adjacent ordinary/CDATA text, exact/one-less budgets,
equal node charges and cancellation at first, second and final node-charge
positions. Successful documents retain kinds, names, prefixes, values,
parents, children, attributes, namespace declarations, order and locations.
All three constructors produce exact unchanged W3C `for-004` output. This does
not yet prove typed-ID, effective-whitespace-view, qualified-namespace,
concurrent reuse or overlapping-generation parity for the candidate.

Both candidates are test-only. Production still uses ordinary growth; no public
API, unsafe surface, dependency, host policy or corpus numerator changes.

## Reproduction and scope

```text
cargo test -p fastxslt --all-features presizing -- --nocapture
cargo test -p fastxslt --all-features freeze_candidate -- --nocapture
cargo test -p fastxslt --release --all-features measure_node_presized_allocations -- --ignored --nocapture --test-threads=1
cargo test -p fastxslt --release --all-features measure_node_freeze_allocations -- --ignored --nocapture --test-threads=1
cargo test -p fastxslt --release --features workbench measure_node_presized_latency -- --ignored --nocapture --test-threads=1
```

Three fresh pre-sizing allocation runs reproduce identical observations. One
new freeze replay reproduces the original six-shape observations and adds the
near-full case. Source generation is outside the allocator scope. Controlled
XML parsing, control creation, XDM construction, reservation or shrinking, and
the final document Arc are inside; the completed Arc remains live afterward.
Source admission, compilation, transformation and adapter work are excluded.
These are requested allocation bytes, not physical copy counts, allocator
overhead, RSS or a hard process cap.

The new shape has 127 elements, each with 31 attributes: 4,066 live nodes in
4,096 reference slots. It is retained in the comparison harness only; the
historical six-shape anatomy and fixture denominators are unchanged.

## Retention and peak

Pre-sizing and freezing have equal final retained requested bytes in every
tested shape. The local allocator yields node capacity equal to live count;
this is an observation, not a portable allocator contract.

| Shape | Reference retained bytes | Candidate retained bytes | Reference peak bytes | Presized peak bytes | Peak reduction |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wide | 1,183,273 | 911,961 | 2,358,867 | 1,808,939 | 23.3% |
| Deep | 141,844 | 78,852 | 346,725 | 224,373 | 35.3% |
| Attribute-heavy | 549,777 | 312,193 | 844,147 | 435,843 | 48.4% |
| Text-heavy | 331,793 | 300,545 | 398,247 | 335,311 | 15.8% |
| Namespace-heavy | 129,041 | 97,793 | 230,363 | 167,902 | 27.1% |
| Low-repetition | 593,973 | 467,493 | 1,186,395 | 921,641 | 22.3% |
| Near-full attributes | 1,173,950 | 1,166,510 | 1,897,584 | 1,559,444 | 17.8% |

| Shape | Reference allocation requests | Presized allocation requests | Reference total requested bytes | Presized total requested bytes |
| --- | ---: | ---: | ---: | ---: |
| Wide | 19,053 | 19,043 | 4,300,465 | 3,014,337 |
| Deep | 1,335 | 1,328 | 561,674 | 372,698 |
| Attribute-heavy | 7,724 | 7,715 | 1,714,297 | 969,801 |
| Text-heavy | 489 | 483 | 466,041 | 372,297 |
| Namespace-heavy | 7,342 | 7,336 | 531,349 | 437,605 |
| Low-repetition | 9,778 | 9,769 | 2,170,999 | 1,537,607 |
| Near-full attributes | 27,478 | 27,468 | 4,697,005 | 3,674,749 |

Retained allocation cardinality is unchanged. Relationship capacities, payloads,
namespace declarations and identity storage remain unchanged. Earlier
reservation could increase coexistence with parser buffers in other workloads;
these seven results do not rule that out.

## Freeze counterexample

On near-full attributes, freeze peak increases from 1,897,584 to 2,182,206
requested bytes, despite only 7,440 fewer retained bytes. Total requested bytes
increase from 4,697,005 to 5,705,373, with one additional resize request. The
original six shapes still show unchanged freeze peaks. Thus unchanged peak was
a fixture result, not a property of freezing.

## Preparation latency

Three fresh release processes omit the allocation-observation feature. Each
shape warms both lanes 32 times, then records 1,001 paired samples with
alternating lane order. The first clock includes parsing, XDM construction and
Arc publication. The second also includes node-count validation and document
release. Controls are created before the clock. Fixture order is fixed and
process windows last 12.9-15.9 seconds; convergence and background host load are
not controlled. Compilation, admission and transformation are outside timing.

Values below are per-process medians in microseconds, reference / presized.
They are paired preparation observations, not full transform or ASP.NET gains.

| Shape | Run | Preparation | Preparation and release |
| --- | ---: | ---: | ---: |
| Wide | 1 | 1,554.9 / 1,333.1 | 1,725.3 / 1,468.4 |
| Wide | 2 | 1,619.7 / 1,406.7 | 1,783.2 / 1,557.8 |
| Wide | 3 | 1,347.4 / 1,174.9 | 1,489.4 / 1,297.5 |
| Deep | 1 | 107.8 / 101.5 | 118.4 / 110.7 |
| Deep | 2 | 133.2 / 125.6 | 146.1 / 137.4 |
| Deep | 3 | 110.5 / 103.3 | 122.6 / 112.1 |
| Attribute-heavy | 1 | 770.5 / 746.2 | 824.3 / 800.5 |
| Attribute-heavy | 2 | 751.6 / 727.9 | 804.2 / 777.2 |
| Attribute-heavy | 3 | 515.1 / 505.4 | 550.6 / 543.7 |
| Text-heavy | 1 | 179.2 / 173.7 | 184.8 / 179.2 |
| Text-heavy | 2 | 181.8 / 179.2 | 187.7 / 184.5 |
| Text-heavy | 3 | 145.4 / 142.6 | 149.2 / 146.0 |
| Namespace-heavy | 1 | 433.2 / 415.6 | 459.1 / 445.5 |
| Namespace-heavy | 2 | 637.9 / 630.9 | 679.2 / 673.2 |
| Namespace-heavy | 3 | 381.5 / 382.0 | 405.2 / 406.1 |
| Low-repetition | 1 | 883.2 / 869.8 | 971.5 / 953.7 |
| Low-repetition | 2 | 841.9 / 819.0 | 923.0 / 896.2 |
| Low-repetition | 3 | 557.5 / 550.0 | 613.4 / 606.5 |
| Near-full attributes | 1 | 3,112.2 / 3,026.9 | 3,358.5 / 3,227.1 |
| Near-full attributes | 2 | 2,426.0 / 2,184.9 | 2,640.7 / 2,342.5 |
| Near-full attributes | 3 | 2,152.5 / 2,050.5 | 2,357.4 / 2,204.2 |

Wide preparation medians improve 12.8-14.3% within each run; namespace-heavy
preparation ranges from a 4.1% reduction to a 0.1% increase. Effects are
shape-dependent and small gains remain vulnerable to noise. No warm evaluator
improvement or reuse break-even follows from these preparation-only clocks.

## Validation and cohesion

At the allocation-comparison checkpoint, workspace verification passed with
1,515 core tests and 44 manual probes ignored,
strict Clippy, formatting, documentation, conformance checks, links and unsafe
surface enforcement. The official WASM target builds; that is not a WASM
measurement of the candidate. Existing incremental-cache hard-link warnings
remain environmental and were not suppressed.

The 425-line comparison child owns only node-capacity parity and measurement,
using its 227-line parent's shape fixtures without sibling-internal access.
The 1,169-line XDM owner retains the candidate reservation beside the same
constructor: splitting it would duplicate construction or spread private
capacity mutation rather than establish a new semantic owner. Retain this
cohesive test-only seam under ADR-0004; reopen at a production decision or new
layout responsibility. At that checkpoint the preparation fixture directory had
four direct files, with no new density pressure. No crate, public type or
alternate engine emerges.

## Semantic and lifecycle follow up

The [capacity semantic parity tests](../../crates/fastxslt/src/runtime/prepared_input_experiment/capacity_semantic_parity_tests.rs)
now compare growth, freeze and pre-sized construction through a bounded internal
DTD fixture with two typed IDs, default attributes, coalesced CDATA text,
comments, a processing instruction, namespace shadowing/default undeclaration,
and `xml:space="preserve"`. Production DTD denial is unchanged.

Visible navigation, string values, ordering, source spans and ID lookup agree
with growth construction. Candidate whitespace views agree with the complete
safe derivation and retain their own prepared origin/storage. Derived namespace
occurrences preserve prefix/value/provenance, parent relationships and charges;
they retain identity across a view but do not alias an independently prepared
equal-byte source. Binding ceilings, exact/one-less view budgets and first/final
view cancellation retain the same outcomes.

Real stripping and preserving stylesheets select `id('alpha')/@key` and count
root text children. Exact outputs are `<out>alpha|0</out>` and
`<out>alpha|3</out>`, with equal charges across all ten work domains.
Terminating-message, exhausted XPath-visit and cooperative instruction-cancel
failures compare as complete structured failures, not display-only text.
Independent retry after controlled failure still returns the exact result.

Copying uses the supported `/r/item[1]` path and verifies the defaulted attribute
and copied namespace payload. Result serialization remains exact after dropping
both candidate source and compiled-program owners. The initial `xsl:copy-of`
selection using `id()` was explicitly rejected with `FXXP1003`; this experiment
does not admit that unsupported language combination.

Four threads execute 128 independently controlled transforms, alternating
stripping and preserving programs over one pre-sized source. A replacement
source/program remains live and executes 33 preserving transforms. A barrier
holds old leases while their publishing owners are dropped; weak references
prove those old source/program owners survive their leases and are released
after joining. Equal-byte replacement sources have distinct qualified identity.
This proves overlapping ownership and concurrent reuse, not achieved occupancy,
host-generation publication policy or performance scaling.

All three new focused tests pass. Workspace gates pass with 1,518 core tests and
44 ignored probes, and the official WASM build check passes. The 403-line
test-only module owns semantic/lifetime conservation rather than anatomy or
timing and accesses existing engine owners, not sibling-test internals. Its
1,138-line preparation parent adds only module registration and retains its
existing preparation responsibility; retain that seam under ADR-0004. Five
direct files in the preparation test directory introduce no density pressure.
There is still no production candidate selection or new conformance pass.

## Remaining adoption evidence

The [subsequent complete-source lifecycle comparison](ar-0027-capacity-single-use-lifecycle-2026-10-02.md)
measures direct Rust single-use and eight-reuse samples and scoped construction
release. It retains timing reversals rather than claiming a universal win.
Representative host measurements, reuse break-even, wider path,
temporary-tree and sealed-external-DTD parity, cancellation-observation gaps,
copied-byte attribution and native, isolated and WASM consumer measurements
remain open. Relationship-vector
capacity is a separate next experiment, not bundled into this candidate.
