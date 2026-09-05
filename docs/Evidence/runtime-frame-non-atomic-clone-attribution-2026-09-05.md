# Runtime-Frame Non-Atomic Clone Attribution

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `c2ec252` plus the ignored measurement probe described here |
| Status | Synthetic pressure confirmed; no representation change admitted |
| Related review | [Performance optimization review](../Reviews/performance-optimization-review-2026-09-04.md) |
| Governing review | [AR-0013](../Architectural%20Reviews/AR-0013-prepared-representation-and-data-layout-audit.md) |

## Question

Does the `RuntimeVariables` clone at `execute_sequence` perform material deep
copying outside the already accepted atomic-variable copy-on-write frame?

There is one explicit complete `RuntimeVariables` clone call site in the
runtime: `execute_sequence`. The atomic map is an `Arc<BTreeMap<...>>` under
ADR-0014 and clones without allocation. Atomic-sequence, source-node,
temporary-tree, and local-shadowing collections still derive complete clone
behavior.

## Fixture

The ignored release probe constructs valid invocation-private frames with 0,
16, 64, or 256 disjoint names in each non-atomic value kind. Every sequence,
source-node selection, and temporary tree contains eight items. Names are
disjoint because the real binder permits only one value kind for a given local
name. Each field and the complete frame are cloned independently under the
allocation observer; five timing samples report their median.

This deliberately pressures clone mechanics. It is not a claim that a normal
stylesheet retains 768 simultaneous local bindings or eight-item temporary
trees under every name.

## Observation

| Bindings per kind | Complete clone | Complete allocations/bytes | Atomic sequences | Source nodes | Temporary trees | Local bindings |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 0.034 us | 0 / 0 | 0.001 us / 0 B | 0.007 us / 0 B | 0.009 us / 0 B | 0.008 us / 0 B |
| 16 | 24.162 us | 426 / 33,652 | 7.275 us / 7,062 B | 1.294 us / 2,790 B | 8.725 us / 22,070 B | 2.335 us / 1,730 B |
| 64 | 82.334 us | 1,692 / 128,404 | 28.920 us / 26,598 B | 5.205 us / 9,270 B | 39.924 us / 85,574 B | 10.165 us / 6,962 B |
| 256 | 334.033 us | 6,774 / 524,188 | 107.496 us / 109,890 B | 22.674 us / 39,090 B | 146.172 us / 346,850 B | 32.215 us / 28,358 B |

At 256 bindings per kind, temporary trees account for about 66.2% of requested
clone bytes and atomic sequences for about 21.0%. Source-node vectors and the
local-binding set account for about 7.5% and 5.4%, respectively. The shared
atomic map remains allocation-free.

The empty-frame result is equally important: the complete clone requests no
allocation and measured about 0.034 us. This does not identify frame cloning as
a current cost for `for-004` or other workloads whose non-atomic local maps are
empty.

## Disposition

No production representation is changed. The probe establishes a conditional
cost model, not consumer-visible pressure. Before extending copy-on-write or
overlay behavior beyond atomic values, a real compiled workload must retain
non-atomic sequences, source-node selections, or temporary trees across nested
sequence execution and show:

- the number of `execute_sequence` clones by frame population;
- the contribution to total transform latency and peak retained memory;
- mutation frequency after cloning and the cost of detachment;
- exact semantic, diagnostic, budget, cancellation, and shadowing parity; and
- a host-visible improvement rather than only a synthetic clone-loop win.

Temporary trees are the first field to investigate if such a workload supplies
pressure. This evidence does not authorize sharing them across invocations,
workers, snapshots, or generations.
