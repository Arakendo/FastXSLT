# Private Prepared-Reuse Shape Timing Probe

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Fixture | `corpus/golden/built-in-template-rules` |
| Shape width | 8 logical source identities or 8 compiled stylesheet identities |
| Toolchain | Rust 1.95.0, x86_64-pc-windows-msvc, LLVM 22.1.2 |
| Command | `cargo test --release -p fastxslt measures_multi_source_and_multi_stylesheet_reuse_shapes -- --ignored --nocapture` |
| Claim | Private local reuse-shape measurement; no cache or host-performance claim |

## Method

The probe admits eight equal-byte source resources under distinct logical
identities and eight equal-byte stylesheet resources under distinct logical
identities. It prepares every source and compiles every stylesheet before
timing. A correctness check requires direct and prepared execution to serialize
identically.

Each run takes seven samples. A sample executes 1,000 iterations of each
eight-operation shape and reports median nanoseconds per transform:

1. one compiled stylesheet over eight sources, parsing and constructing XDM for
   each invocation;
2. the same stylesheet over the eight explicitly prepared sources;
3. eight compiled stylesheets over one source, parsing and constructing XDM for
   each invocation; and
4. the same eight stylesheets over one explicitly prepared source.

Compilation, preparation, admission, and output comparison occur outside timed
windows. The optional `allocation-observation` feature remains disabled, so its
global allocator wrapper is not linked into this timing binary.

## Results

| Run | Multi-source direct | Multi-source prepared | Ratio | Multi-style direct | Multi-style prepared | Ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2,705.9 ns | 845.3 ns | 3.20× | 2,739.4 ns | 832.3 ns | 3.29× |
| 2 | 2,776.1 ns | 833.0 ns | 3.33× | 2,762.3 ns | 804.4 ns | 3.43× |
| 3 | 2,675.0 ns | 839.8 ns | 3.19× | 2,647.9 ns | 821.9 ns | 3.22× |

## Interpretation

Prepared reuse has similar value in both demonstrated relationship shapes. On
this tiny transform, avoiding XML parse and XDM construction dominates whether
the workload varies source identity or compiled stylesheet identity.

The test preserves distinct source identities and separately compiled program
allocations; it does not content-address or merge either resource class.

## Limitations

- Every source and stylesheet has equal bytes; only logical identity varies.
- The transform is tiny and not confirmed by an intended consumer.
- The probe is single-threaded and warm, with no first-access contention,
  scheduling, cache miss, eviction, or reconstruction.
- Preparation and compilation cost, retained/peak memory, file import, ASP.NET,
  FFI, result transfer, and process working set are outside these timed paths.
- Ratios do not select eager, lazy, single-flight, eviction, or public lifecycle
  defaults.
