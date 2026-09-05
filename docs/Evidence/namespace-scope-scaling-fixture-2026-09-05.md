# Namespace-Scope Scaling Fixture

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `d5a54aa` plus the ignored measurement fixture described here |
| Status | Pressure confirmed; namespace-stack implementation not yet admitted |
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

## Next experiment

Compare the complete serializer with a safe invocation-owned scope stack that:

- appends shadowing bindings and restores a checkpoint after each element;
- resolves only the currently active binding for a prefix;
- preserves default undeclarations, namespaced-attribute prefix selection,
  multiple prefixes for one URI, and sibling restoration;
- keeps complete byte-for-byte serialization as the differential oracle; and
- reports allocation/time scaling by depth and active binding count.

Do not retain a stack unless it improves this fixture without worsening shallow
ordinary results, and do not infer a result-tree namespace representation from
serializer evidence alone.
