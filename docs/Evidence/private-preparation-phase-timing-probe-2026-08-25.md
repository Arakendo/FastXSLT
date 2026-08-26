# Private Preparation-Phase Timing Probe

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Fixture | `corpus/golden/built-in-template-rules/input.xml` |
| Source size | 55 bytes |
| Toolchain | Rust 1.95.0, x86_64-pc-windows-msvc, LLVM 22.1.2 |
| OS | Microsoft Windows 10.0.26200 |
| CPU identifier | AMD64 Family 25 Model 97 Stepping 2, AuthenticAMD |
| Command | `cargo test --release -p fastxslt measures_preparation_phase_time_separately -- --ignored --nocapture` |
| Claim | Private local phase measurement; no lifecycle or host-performance claim |

## Method

Each run takes seven samples of 10,000 iterations and reports the median
nanoseconds per iteration for two independently timed phases:

- XML parsing from the same admitted byte slice into an owned event document;
- XDM construction from previously parsed event documents.

The XDM inputs are created before its timed window so XML parsing is excluded.
Both loops consume results through `black_box`; each produced phase object is
dropped within the timed loop, so the reported values include ordinary cleanup
for that object. Preparing the vector of XDM inputs is not timed.

## Results

| Run | XML parse median | XDM construct-and-drop median |
| --- | ---: | ---: |
| 1 | 1,133.1 ns | 921.5 ns |
| 2 | 1,123.9 ns | 863.4 ns |
| 3 | 1,133.3 ns | 884.2 ns |

## Interpretation

Both phases are material contributors for this tiny syntax-light source. The
combined ranges are consistent with the earlier complete direct-path probe,
but they must not be added and presented as an exact decomposition: the probes
have different loop structure, allocation lifetime, cleanup, and cache state.

This closes only the local ability to observe parse and XDM construction time
separately. It does not select eager preparation, lazy preparation,
single-flight, eviction, or a public handle lifecycle.

## Limitations

- This is an ignored release-mode microprobe, not a CI performance threshold.
- The XDM input vector deliberately changes memory and cache pressure relative
  to one parse-then-build invocation.
- The source is 55 bytes and is not confirmed as consumer-representative.
- No allocator-inclusive retained or peak memory is measured.
- No compilation, transform, serialization, concurrency, ASP.NET, FFI,
  transport, file import, or security-tool interaction is included.
