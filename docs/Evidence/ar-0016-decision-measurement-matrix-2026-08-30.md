# AR-0016 Decision Measurement Matrix

Date: 2026-08-30

## Scope

This record compares AR-0016's complete derived-document reference with the
invocation-owned visibility view for the exact admitted
`xsl:strip-space elements="*"` policy. It measures construction, total warm
execution, view latency, four-thread throughput, and allocator-requested
retained/peak bytes. It is decision evidence for one private representation,
not a general XSLT performance claim or public benchmark.

## Environment and commands

- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Target: `x86_64-pc-windows-msvc`
- LLVM: 22.1.2
- OS: Microsoft Windows NT 10.0.26200.0
- Processor identifier: AMD64 Family 25 Model 97 Stepping 2
- Logical processors visible to the process: 16

Timing matrix:

```text
cargo test --release -p fastxslt --lib measures_whitespace_representation_matrix -- --ignored --nocapture
```

Allocator-requested memory:

```text
cargo test --release -p fastxslt --lib --features allocation-observation measures_whitespace_representation_allocations -- --ignored --nocapture
```

Each timing cell is the median of seven batches. The batch size varies from
250 to 4,000 invocations according to source size. Each timed transform builds
its invocation-owned effective representation and executes the same compiled
value-producing stylesheet. View p50/p95/p99 values come from 1,001 individual
invocations. Concurrent throughput uses four scoped workers with independent
invocation state over the same immutable source and compiled stylesheet.

## Source-shape results

| Workload | Nodes | Reference construction | View construction | Construction advantage | Preserve total | Reference total | View total | Total advantage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Small, whitespace-heavy | 51 | 4,960 ns | 830 ns | 5.98x | 854 ns | 5,940 ns | 1,893 ns | 3.14x |
| Medium, whitespace-heavy | 1,503 | 189,675 ns | 21,085 ns | 9.00x | 11,263 ns | 221,753 ns | 42,334 ns | 5.24x |
| Medium, whitespace-light | 1,002 | 116,934 ns | 20,468 ns | 5.71x | 7,701 ns | 125,820 ns | 29,359 ns | 4.29x |
| Large, whitespace-heavy | 6,003 | 999,833 ns | 105,212 ns | 9.50x | 53,325 ns | 1,257,739 ns | 150,678 ns | 8.35x |
| Deep, whitespace-heavy | 150 | 20,179 ns | 5,589 ns | 3.61x | 2,674 ns | 17,421 ns | 6,367 ns | 2.74x |

The preserving baseline performs no effective-view construction. The view's
total cost is 2.22x to 3.81x that baseline on these deliberately stripping
workloads. This is the cost of applying stylesheet-dependent semantics; it is
not paid by a preserving stylesheet. Against the complete reference, the view
is already ahead when construction and one execution are combined, so there is
no retained-view reuse threshold to amortize between these invocation-owned
candidates.

## Latency and concurrency

| Workload | View p50 | View p95 | View p99 | Reference x4/s | View x4/s | View throughput advantage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Small, whitespace-heavy | 1.6 us | 2.7 us | 3.2 us | 457,221 | 1,346,983 | 2.95x |
| Medium, whitespace-heavy | 31.6 us | 52.2 us | 66.4 us | 10,215 | 74,021 | 7.25x |
| Medium, whitespace-light | 26.4 us | 41.1 us | 53.5 us | 16,471 | 88,334 | 5.36x |
| Large, whitespace-heavy | 135.1 us | 213.8 us | 244.2 us | 2,046 | 18,150 | 8.87x |
| Deep, whitespace-heavy | 6.2 us | 9.2 us | 10.3 us | 116,133 | 293,591 | 2.53x |

The view remains ahead under four concurrent invocations on every source
shape. No shared mutable cache, lock, or cross-generation state is involved;
each worker constructs and drops its own view over shared immutable prepared
node storage.

## Allocator-requested memory

The memory probe uses the large 6,003-node whitespace-heavy source.

| Phase | Requested bytes total | Retained bytes after construction | Peak requested bytes | Peak live allocations |
| --- | ---: | ---: | ---: | ---: |
| Complete-reference construction | 4,293,296 | 2,262,648 | 3,214,912 | 14,010 |
| Visibility-view construction | 112,876 | 16,116 | 32,408 | 4 |
| Preserve total invocation | 33,758 | 0 | 25,091 | 6 |
| Complete-reference total invocation | 4,310,644 | 0 | 3,214,912 | 14,015 |
| Visibility-view total invocation | 130,224 | 0 | 32,408 | 8 |

The view construction retains about 140.4 times fewer requested bytes and
peaks about 99.2 times lower than the complete clone. End-to-end peak requested
bytes retain the same approximately 99.2-times difference. Both total
invocations return to zero retained requested bytes after their result is
dropped; neither candidate creates a retained cache.

`allocation-counter` observes allocator-requested bytes in this process. These
figures are not process working set, RSS, or a host-boundary memory guarantee.

## Decision relevance

- Semantic parity remains independently established by the complete reference
  and unchanged `mode-1301` case.
- The view wins total invocation time for every tested size/shape, including a
  single construction-plus-execution lifecycle repeated without retained view
  reuse.
- The advantage persists and generally widens under four concurrent workers.
- Requested retained and peak memory are materially lower, not merely a
  container-capacity estimate.
- A preserving stylesheet bypasses view construction entirely.

These observations are sufficient to select the invocation-owned visibility
view for the exact admitted strip-all policy while retaining the complete clone
as a test oracle. They do not select generalized whitespace matching,
`xsl:preserve-space`, import precedence, `xml:space`, typed whitespace, a public
source-view abstraction, or cross-invocation caching.
