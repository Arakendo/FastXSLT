# Namespace-Scope Scaling Fixture

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `d739bcc` plus the scoped-stack implementation and comparison described here |
| Status | Safe private scoped stack retained; complete clone remains the differential oracle |
| Related review | [Performance optimization review](../Reviews/performance-optimization-review-2026-09-04.md) |
| Governing review | [AR-0013](../Architectural%20Reviews/AR-0013-prepared-representation-and-data-layout-audit.md) |

## Question

Does a namespace-heavy result show enough scaling pressure to justify comparing
the serializer's per-element scope cloning with a safe invocation-owned
namespace stack?

The ignored release fixture compiles and executes a real stylesheet containing
8, 24, or 48 nested literal result elements. Every level introduces eight new
prefix bindings and a namespaced attribute, so active scope grows with depth.
The source, stylesheet, result-node, and serialized-byte limits remain explicit.

## Observation

One release run reported medians of five timing samples. Allocation values are
single observed executions or serializations.

| Depth | Result bytes | Execution | Serialization | Execution allocations/bytes | Serialization allocations/bytes | Serializer peak live bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 3,191 | 18.046 us | 102.596 us | 651 / 35,247 | 1,195 / 66,383 | 43,743 |
| 24 | 9,867 | 146.111 us | 1,136.987 us | 5,019 / 235,447 | 9,709 / 541,296 | 341,300 |
| 48 | 19,947 | 565.021 us | 7,269.868 us | 19,251 / 865,975 | 37,838 / 2,094,320 | 1,300,212 |

From depth 24 to 48, serialized output grew about 2.0x while semantic execution
grew about 3.9x, serialization grew about 6.4x, and allocation requests in both
phases grew about 3.8-3.9x. This is directional single-machine evidence, not a
general engine benchmark.

## Interpretation

The serializer's current `element_namespace_scope` clones the inherited scope,
then performs linear searches and prefix-based retention for every binding.
The measured shape supplies the pressure required by the review before a stack
comparison.

The semantic-execution growth is equally important. Literal result nodes retain
namespace vectors before serialization, so an invocation-owned serializer
stack cannot by itself remove all observed namespace cost. The next experiment
must attribute result-tree namespace materialization separately from serializer
scope composition rather than reporting one combined “namespace optimization.”

## Scoped-stack comparison

The admitted candidate is a safe invocation-owned scope stack that:

- appends shadowing bindings and restores a checkpoint after each element;
- resolves only the currently active binding for a prefix;
- preserves default undeclarations, namespaced-attribute prefix selection,
  multiple prefixes for one URI, and sibling restoration;
- keeps complete byte-for-byte serialization as the differential oracle; and
- leaves XHTML5 default-namespace normalization on the complete owned path
  because that mode synthesizes transient namespace bindings.

An initial owned-string/`HashMap<Option<String>, _>` implementation was rejected
before admission. At depth 48 it was about 24% slower than the complete
baseline and allocated about 65% more bytes: it had replaced scope cloning with
different ownership traffic rather than removing it.

The second implementation borrows immutable result-tree prefix and namespace
strings. Each distinct prefix receives one invocation-local slot, and the
push/restore history contains only slot and prior-index scalars. A paired release
run measured both serializers against the same retained semantic result:

| Workload | Scoped stack | Complete clone | Speedup | Scoped allocations/bytes/peak | Complete allocations/bytes/peak |
| --- | ---: | ---: | ---: | ---: | ---: |
| ordinary `for-004`, 500 items | 0.251 us | 0.269 us | 1.07x | 4 / 125 / 110 | 4 / 125 / 110 |
| namespace depth 8 | 38.149 us | 108.163 us | 2.84x | 125 / 85,219 / 52,931 | 1,211 / 66,447 / 43,771 |
| namespace depth 24 | 201.071 us | 1,114.088 us | 5.54x | 299 / 591,104 / 376,020 | 9,757 / 541,516 / 341,405 |
| namespace depth 48 | 1,023.589 us | 7,371.836 us | 7.20x | 548 / 2,247,616 / 1,444,274 | 37,934 / 2,094,780 / 1,300,437 |

The candidate removes up to 98.6% of allocation requests, but does not improve
every memory dimension. At depth 48 it allocates about 7.3% more total bytes and
raises measured peak live serializer bytes about 11.1%. That cost is recorded
rather than hidden; the pathological fixture's 7.20x time improvement and the
absence of an ordinary shallow allocation regression justify retaining the
private implementation.

Focused differential tests preserve exact bytes and serialized-byte charges
across prefix shadowing, two prefixes for one URI, default undeclaration,
sibling restoration, byte-limit failure, and cancellation. The complete safe
implementation remains selectable only in tests and for the existing XHTML5
normalization path.

The retained responsibility lives in the private 392-line
`serialization/namespace_scope.rs` module. Serialization still owns output
method behavior, escaping, work charging, and diagnostics; the extracted module
owns only namespace entry/exit, active-prefix lookup, declarations, and its
complete reference. It introduces no crate, public provider, runtime-state, or
host dependency.

## Disposition

The serializer stack is retained. It does not admit a public namespace API,
unsafe code, cross-invocation state, or a result-tree namespace representation.
Semantic result construction still exhibits separate superlinear namespace
retention and remains a future, independently evidenced experiment.
