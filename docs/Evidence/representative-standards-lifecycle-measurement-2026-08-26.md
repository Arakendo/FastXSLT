# Representative Standards Lifecycle Measurement

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Workloads | Native XSLT30 `for-004` and `castable-004` |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Toolchain | Rust 1.95.0, x86_64-pc-windows-msvc, LLVM 22.1.2 |
| Timing command | `cargo test --release -p fastxslt measures_representative_standards_lifecycle -- --ignored --nocapture` |
| Allocation command | `cargo test --release -p fastxslt --features allocation-observation measures_representative_standards_preparation_allocations -- --ignored --nocapture` |
| Claim | Private local Rust lifecycle observation; no cache, ASP.NET, or application-performance claim |

## Method

The probe imports each unmodified pinned source and stylesheet into its own
bounded memory-resident snapshot. Before timing, direct and prepared execution
must serialize identically. Each timing result is the median of seven samples of
1,000 iterations.

The independently timed paths are:

1. XML parsing from retained source bytes;
2. XDM construction and drop from inputs parsed before the timed window;
3. stylesheet compilation from a retained snapshot resource;
4. compiled/direct execution: parse, construct XDM, execute, and serialize with
   one compiled stylesheet retained outside timing;
5. compiled/prepared execution: prepared lookup, execute, and serialize with
   compiled and prepared state retained outside timing; and
6. compile-each execution: compile, parse, construct XDM, execute, and serialize
   within every iteration.

The phases use different allocation lifetimes and cache states. Independent
parse, XDM, and compile numbers must not be added or subtracted as an exact
decomposition of either complete execution path.

## Timing results

Ranges below cover three complete local runs.

| Workload | Parse | XDM construct/drop | Compile | Compiled/direct | Compiled/prepared | Prepared ratio | Compile each | Compiled ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `for-004` | 4,692.7–6,261.3 ns | 2,674.8–5,534.1 ns | 7,257.7–9,027.3 ns | 8,435.4–14,245.9 ns | 1,649.4–2,046.0 ns | 4.98–6.96× | 15,907.8–21,029.9 ns | 1.48–1.91× |
| `castable-004` | 5,039.3–7,107.8 ns | 4,455.5–8,175.9 ns | 46,485.4–58,658.7 ns | 29,742.8–33,642.7 ns | 20,020.6–27,875.3 ns | 1.21–1.50× | 77,288.6–84,680.4 ns | 2.52–2.71× |

Prepared reuse removes the same parse/XDM work in both cases, yet its ratio is
far smaller when XPath/XSLT execution dominates. Conversely, the larger
`castable-004` stylesheet makes compiled reuse more valuable. A single “reuse
is N times faster” number would conceal the workload composition.

## Retention and peak results

The optional exact-pinned allocator counter surrounds only explicit source
preparation on the calling thread.

| Workload | Raw source | Parsed representation capacity | XDM nodes | XDM-owned capacity | Allocator-requested retained | Allocator-requested peak |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `for-004` | 216 B | 5,720 B | 23 | 8,314 B | 9,124 B | 15,943 B |
| `castable-004` | 425 B | 7,708 B | 39 | 16,592 B | 17,412 B | 30,722 B |

Prepared reuse trades repeated work for retained memory substantially larger
than the source bytes. The parsed capacity is a completed-parse phase
observation, not memory retained alongside the sealed XDM. The allocator peak
includes co-resident construction allocations within the measured closure.

## Limitations

- These standards cases exercise implemented semantics but are still small and
  are not confirmed consumer workloads.
- Measurements are warm, single-threaded, Rust-only, and local to one Windows
  host. They exclude ASP.NET, FFI, scheduling, result transfer, file import, and
  process working set.
- The allocation counter reports requested bytes, not allocator metadata,
  fragmentation, snapshot admission, compiled stylesheet retention, runtime
  transients, or other threads.
- The probe establishes no eager/lazy preparation, single-flight, eviction,
  cache-size, worker-count, or public handle default.
