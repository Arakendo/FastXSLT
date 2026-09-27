# WASM Browser-Boundary Feasibility and Benchmark Baseline

| Field | Value |
| --- | --- |
| Date | 2026-09-24 |
| Status | Exploratory evidence; not a supported target or performance claim |
| Review informed | [AR-0015](../Architectural%20Reviews/AR-0015-wasm-embedding-profile-and-host-boundary.md) |
| Target | `wasm32-unknown-unknown` |
| Host used | Node.js 22.23.2 with private `wasm-bindgen` Node glue |
| Rust toolchain | `rustc 1.95.0`; `cargo 1.95.0` |
| Binding tool | `wasm-bindgen-cli 0.2.126` |

## Question

Can the existing safe FastXSLT semantic engine compile to WebAssembly, retain one
compiled/prepared engine across calls, consume only explicitly admitted bytes,
preserve representative structured controls and diagnostics, and expose enough
phase information for a future browser benchmark?

This tranche does not establish browser compatibility. Node is a convenient
JavaScript/WASM host for the first boundary probe; browser packaging, event-loop
behavior, Web Workers, CSP, download size, and browser memory ceilings remain
unmeasured.

## Method

The unpublished `fastxslt-wasm-workbench` crate uses `#![forbid(unsafe_code)]`
and depends on the ordinary `fastxslt` crate with its private `workbench`
feature. The adapter:

- copies source, principal stylesheet, and optional dependency bytes into owned
  Rust storage;
- creates the existing sealed resource snapshot;
- compiles the stylesheet and prepares the principal source once;
- retains the resulting `ExperimentalEngine` inside one WASM instance;
- executes one invocation at a time through the same engine path;
- returns owned result bytes or bounded structured diagnostic fields; and
- exposes read-only retained-capacity and WASM linear-memory observations.

The JavaScript harness runs:

1. a synthetic relative `xsl:include` whose dependency exists only in the
   sealed in-memory resource set;
2. an instruction-budget failure and successful reuse of the same engine;
3. malformed XML classification;
4. the unchanged upstream XSLT30 `for-004.xsl` stylesheet against generated
   5-, 50-, and 500-item sources; and
5. a warmed host-to-WASM byte-copy calibration.

The benchmark command was:

```powershell
./scripts/measure-wasm-workbench.ps1 -Iterations 1000
```

Generated glue and the built module remain under the gitignored
`.workbench/wasm-bindgen/` directory.

## Semantic and control observations

| Probe | Observation |
| --- | --- |
| Sealed relative include | Exact result `<out>sealed</out>` with no filesystem or network fallback |
| Zero XSLT-instruction budget | `FXCT0002` / `limit` |
| Reuse after budget failure | The same retained engine completed the next request exactly |
| Malformed source XML | `FXXD0002` / `invalid` |
| XSLT30 `for-004` | Exact expected serialized result at every tier |
| Active invocations | One sequential invocation in one instance |

This is representative parity evidence, not the complete AR-0015 diagnostic or
serialization matrix. The malformed-source projection currently reports no
resource/span fields; that is an observed adapter gap to compare against direct
Rust rather than a new diagnostic contract.

## First performance observation

The release WASM binary generated for the Node binding was **3,081,083 bytes**.
Two fresh Node runs reported module loading of **6.826-10.454 ms** and first
sealed-include engine creation of **9.314-20.430 ms**, including host copies,
snapshot construction, compilation, and source preparation.

| Items per transform | Warm transforms/s, run 1 | Warm transforms/s, run 2 | Mean execution + serialization range | Mean result transfer range | Known retained capacity | Prepared-XDM capacity | Nodes | Linear-memory pages after tier |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 71,053.51 | 109,364.92 | 6.956-10.685 us | 1.288-2.360 us | 9,018 B | 4,800 B | 17 | 26 |
| 50 | 47,594.91 | 68,184.69 | 13.438-19.202 us | 0.837-1.109 us | 43,251 B | 39,027 B | 152 | 29 |
| 500 | 9,344.14 | 14,256.12 | 69.243-105.118 us | 0.563-1.187 us | 330,859 B | 326,629 B | 1,502 | 40 |

These are short exploratory observations on one machine. Their substantial
between-run drift is itself evidence that a convergence and run-rotation method
is required before comparison. They are not a native comparison, browser claim,
regression gate, or publication-quality benchmark. Linear-memory pages are
cumulative instance observations and do not attribute allocator slack or peak
construction memory. The setup timer combines input copies, snapshot creation,
compile, and prepare; later work must split those phases before drawing
boundary-cost conclusions.

## Dependency and licence review

The feasibility crate adds `wasm-bindgen 0.2.126`. Its target dependency graph
adds the following packages beyond FastXSLT's existing engine graph:

| Package | Version | Declared licence |
| --- | ---: | --- |
| `wasm-bindgen` | 0.2.126 | MIT OR Apache-2.0 |
| `wasm-bindgen-macro` | 0.2.126 | MIT OR Apache-2.0 |
| `wasm-bindgen-macro-support` | 0.2.126 | MIT OR Apache-2.0 |
| `wasm-bindgen-shared` | 0.2.126 | MIT OR Apache-2.0 |
| `bumpalo` | 3.20.3 | MIT OR Apache-2.0 |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 |
| `syn` | 2.0.119 | MIT OR Apache-2.0 |

The declared licences are compatible with FastXSLT's MIT distribution. This is
dependency-admission evidence, not a claim that the generated artifact or every
transitive dependency contains no unsafe code. The first-party adapter forbids
unsafe code; a transitive unsafe/tool audit remains open under AR-0015.

## Conclusions and limits

The safe engine and a deliberately private JavaScript-facing adapter compile
and execute successfully on `wasm32-unknown-unknown`. The experiment supports
the architectural premise that WASM can reuse FastXSLT's host-neutral,
presealed, compile-once/prepare-once lifecycle without a second semantic engine
or ambient acquisition authority.

It does not yet justify a supported WASM target or public binding. The following
remain open:

- direct-Rust differential coverage for the full positive/failure matrix;
- deterministic explicit release and generation replacement;
- browser execution and packaging;
- 32-bit length checks and 1/10/100 MiB source admission;
- peak construction and per-owner memory attribution;
- text, HTML, namespace-heavy, and broader XSLT 1.0 coverage;
- cancellation with a credible host signal;
- bounded multi-instance/Web Worker scaling; and
- comparison with native and isolated host lanes under an equivalent workload.
