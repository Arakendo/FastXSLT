# Nested Runtime-Frame Copy-on-Write Experiment

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `f381f8f` plus the measured candidate |
| Status | Safe invocation-owned candidate retained through ADR-0017 |
| Prior evidence | [Non-atomic field attribution](runtime-frame-non-atomic-clone-attribution-2026-09-05.md) |
| Governing review | [AR-0013](../Architectural%20Reviews/AR-0013-prepared-representation-and-data-layout-audit.md) |

## Question

Does complete cloning of non-atomic `RuntimeVariables` maps materially affect a
real compiled nested execution path, and can safe invocation-private
copy-on-write remove that cost without moving it into mutation or changing
semantics?

## Workload and method

The generated stylesheet is compiled by the ordinary engine. Its outer
template creates disjoint local bindings for eight-item atomic sequences,
eight retained source-node identities, and eight-node temporary trees. It then
constructs one or eight nested literal-result elements. Every literal body
enters the normal `execute_sequence` scope path. Test-only bounded observations
record clone count and source-frame population. The complete deep-clone
implementation remains selectable only in tests.

Five release-mode samples report median microseconds. The allocation observer
reports one execution's allocation requests, total requested bytes, and peak
live requested bytes. This is a generated mechanism workload, not a claim about
consumer binding distributions.

## Read-only nested comparison

| Bindings per kind | Depth | Complete median | COW median | Speedup | Complete allocations / total / peak bytes | COW allocations / total / peak bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 1 | 1.026 us | 0.732 us | 1.40x | 23 / 2,203 / 1,499 | 13 / 1,755 / 1,275 |
| 0 | 8 | 6.029 us | 3.346 us | 1.80x | 100 / 11,100 / 5,468 | 55 / 9,084 / 5,244 |
| 4 | 1 | 42.445 us | 17.424 us | 2.44x | 521 / 39,351 / 29,716 | 291 / 27,047 / 17,980 |
| 4 | 8 | 156.531 us | 28.613 us | 5.47x | 2,166 / 132,528 / 115,564 | 333 / 34,376 / 21,948 |
| 16 | 1 | 147.935 us | 85.275 us | 1.73x | 1,999 / 147,615 / 111,980 | 1,103 / 100,899 / 65,831 |
| 16 | 8 | 635.092 us | 82.591 us | 7.69x | 8,306 / 481,676 / 438,712 | 1,145 / 108,228 / 69,800 |

At the largest shape, copy-on-write reduced median time by 87.0%, allocation
requests by 86.2%, total requested bytes by 77.5%, and peak live requested bytes
by 84.1%. The frame observations were identical: nine sequence scopes, eight
populated clones, and 128 cloned-entry opportunities in each value map plus 384
shadow-metadata entries.

## Mutation-at-every-level counter-case

A second compiled stylesheet starts with the same retained bindings but creates
new atomic-sequence, source-node, and temporary-tree bindings inside every one
of eight nested literal scopes. This forces every shared non-atomic map to
detach at every level.

| Initial bindings per kind | Complete median | COW median | Complete allocations / total / peak bytes | COW allocations / total / peak bytes |
| ---: | ---: | ---: | ---: | ---: |
| 4 | 440.348 us | 303.279 us | 4,250 / 256,064 / 221,356 | 4,241 / 255,704 / 220,996 |
| 16 | 730.396 us | 728.864 us | 10,398 / 615,612 / 555,086 | 10,389 / 615,252 / 554,160 |

The largest mutation-heavy path is effectively neutral. It confirms that the
optimization defers rather than avoids required ownership separation, but does
not add a material penalty when every map must detach.

## Conservation

Focused tests establish that parent and child maps initially share one
invocation-owned backing; a child mutation detaches only its affected value kind
and shadow metadata; cross-kind replacement cannot mutate the parent's value;
and the complete oracle owns independent copies of every map. Shared and
complete compiled workloads produce equal semantic results and work-domain
totals. Deterministic cancellation occurs with the same code, category, work
domain, and accepted instruction charge count.

The full engine suite supplies broader shadowing, temporary-tree, concurrent
invocation, corpus, and host-boundary regression coverage.

## Disposition

ADR-0017 retains the safe private representation. It admits neither a public
frame abstraction nor sharing outside one invocation. No current ASP.NET
fixture has the populated nested-frame shape, so this evidence makes no
consumer-wide throughput claim; ordinary host gates serve as non-regression
evidence until a representative consumer supplies this pressure.
