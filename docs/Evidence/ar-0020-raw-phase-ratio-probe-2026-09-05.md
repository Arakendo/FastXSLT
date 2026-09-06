# AR-0020 Raw Preparation-to-Execution Phase Probe

| Field            | Value                                                                                                   |
| ---------------- | ------------------------------------------------------------------------------------------------------- |
| Date             | 2026-09-05                                                                                              |
| Status           | Preliminary topology-selection evidence                                                                 |
| Governing review | [AR-0020](../Architectural%20Reviews/AR-0020-bounded-pre-execution-preparation-pipeline.md)             |
| Code baseline    | `10d2f30` plus the uncommitted AR-0020 reference                                                        |
| Toolchain        | Rust/Cargo 1.95.0; optimized release test build                                                         |
| Stylesheet       | Pinned W3C XSLT30 `for-004` at repository submodule revision `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Sources          | Deterministically generated 5-, 50-, and 500-item `order` documents                                     |
| Samples          | 51 successful complete prepare/execute samples per tier per run; two runs                               |

## Question

Is controlled raw-source preparation large enough relative to transformation to
justify comparing overlapped preparation topologies, or is another queue/thread
likely to move work that is already negligible?

The probe compiles the pinned stylesheet once. Every sample then independently
parses its sealed source, constructs prepared XDM, retains one immutable packet,
executes the transform, serializes it, and checks the expected `<out>N.00</out>`
semantic sentinel. Preparation time includes invocation-control construction,
source lookup, controlled XML parsing, controlled XDM construction, and packet
publication. Execution time includes semantic execution and serialization. It
does not include a queue or cross-thread handoff.

Command:

```powershell
cargo test --release -p fastxslt measures_pinned_for004_raw_phase_ratio --all-features -- --ignored --nocapture
```

## Results

| Items | Known prepared capacity | Preparation median range | Execution median range | Preparation / execution range |
| ----: | ----------------------: | -----------------------: | ---------------------: | ----------------------------: |
|     5 |                 9,039 B |             14.3-16.0 us |             3.2-6.0 us |                    2.38-5.00x |
|    50 |                72,779 B |           100.2-115.6 us |           12.6-14.4 us |                    7.95-8.03x |
|   500 |               598,179 B |           865.7-912.5 us |         133.8-136.0 us |                    6.47-6.71x |

The tiny tier is timer- and warmup-sensitive, as its two execution medians show.
The 50- and 500-item tiers are directionally stable across these two runs:
source preparation is materially larger than execution for this workload.

## Interpretation and boundary

This passes AR-0020's first prerequisite for a topology experiment:
preparation is not negligible. It does **not** establish that a preparation
thread improves end-to-end throughput. Parsing/XDM work may contend for memory
bandwidth, queue transfer may erase a gain, or preparation may outrun execution
and amplify retained memory. The next experiment must compare worker-local,
one-producer, and bounded-producer-pool execution with identical results and
count/byte backpressure.

This is a pinned-stylesheet mechanics probe with generated pressure sources,
not a representative publication workload, host-boundary measurement, or
general FastXSLT performance claim. The existing warm ASP.NET workload remains
the required negative control, and consumer-owned source distributions and
limits remain outstanding.
