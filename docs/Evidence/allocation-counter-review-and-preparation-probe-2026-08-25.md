# Allocation-Counter Review and Preparation Probe

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Package | `allocation-counter` 0.8.1 |
| Cargo checksum | `beb9e990c0a33699f1984d85a6abead615ccc72dd8130bf3e15dcabe2ca149c9` |
| License | MIT OR Apache-2.0 |
| Admission | Exact-pinned development dependency only |
| Command | `cargo test --release -p fastxslt measures_preparation_allocations -- --ignored --nocapture` |
| Claim | Current-thread allocator-request observation; not process or host memory |

## Dependency review

The published 0.8.1 package has no transitive dependencies. Its allocator
surface is one small module containing a `GlobalAlloc` implementation with
`alloc` and `dealloc` forwarding to `std::alloc::System`. It records counters in
thread-local `RefCell` state and installs the wrapper as the test binary's global
allocator.

The dependency contains one `unsafe impl GlobalAlloc` and two unsafe methods.
FastXSLT adds no first-party unsafe code, does not relax `unsafe_code =
"forbid"`, and does not require an ADR-0003 exception for this experiment.
Dependency unsafe remains part of the test-tool trust boundary.

Known limitations retained from source inspection:

- measurements cover allocations on the calling thread only;
- a panic inside `measure` does not restore its nesting depth before unwinding;
- requested `Layout` sizes exclude allocator metadata, size-class rounding,
  fragmentation, thread stacks, mapped pages, and unrelated process memory;
- the wrapper affects the Rust test binary whenever the dev dependency is
  linked; and
- the package declares no Rust version and has no dependencies of its own.

The package is not a runtime dependency and does not propagate to FastXSLT
consumers. Its dual MIT/Apache-2.0 terms are compatible with the repository's
MIT distribution; the exact package and checksum remain recorded in
`Cargo.lock`.

## Measurement boundary

Each probe begins with an already admitted sealed snapshot and an empty private
`PreparedInputBuilder`. The measured closure performs XML parsing, XDM
construction, and insertion of the prepared document and its phase observation
into the builder. The builder remains alive after the closure.

Consequently:

- `bytes_total` is all allocator-requested bytes during explicit preparation;
- `bytes_current` is the net requested allocation retained by the prepared
  builder at closure exit; and
- `bytes_max` is the maximum net requested allocation above the closure's
  starting point while preparation runs.

Snapshot admission and its retained raw bytes occur before measurement.

## Results

Three repeated optimized runs produced identical observations:

| Fixture | Total allocations | Retained allocations | Peak allocations | Total requested | Retained requested | Peak requested |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 87-byte hello source | 40 | 20 | 20 | 5,250 bytes | 2,744 bytes | 3,424 bytes |
| 2,109-byte/100-item source | 644 | 511 | 512 | 208,343 bytes | 64,577 bytes | 130,357 bytes |

The corresponding representation observations remain 938 parsed-phase bytes,
6 XDM nodes, and 1,932 XDM-capacity bytes for hello; and 46,862 parsed-phase
bytes, 202 nodes, and 63,755 XDM-capacity bytes for the generated source.

## Interpretation

The generated source's 130,357-byte construction peak is roughly twice its
64,577-byte retained prepared delta, confirming that retained XDM alone is not
a sufficient construction budget. The close 64,577 allocator-requested versus
63,755 representation-capacity values also show that the existing capacity
observation explains most—but not all—retained preparation allocation for this
fixture.

These exact values are toolchain, representation, identity-length, allocator,
and fixture observations. They do not establish a stable formula, cache size,
host budget, ASP.NET working set, or consumer performance claim.
