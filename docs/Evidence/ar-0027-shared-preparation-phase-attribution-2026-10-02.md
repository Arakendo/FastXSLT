# AR 0027 Shared Preparation Phase Attribution

| Field | Value |
| --- | --- |
| Date | 2026-10-02 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Control | [Direct Rust lifecycle baseline](ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md) |
| Suite revision | XSLT30 `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Workload | Unchanged `for-004` with its unchanged `for03.xml` input, plus first-party synthetic wide sources |
| Build | Local Windows native release, `workbench` feature, without `allocation-observation` |
| Disposition | Phase attribution only; no layout, staging, parser dependency or public contract selected |

The shared preparation path now has a test-only observer around controlled XML
parsing and XDM construction. XML parsing is the larger measured phase in all
four inputs. This identifies preparation costs without replacing the engine,
changing resource authority or treating a parser-only microbenchmark as Rust
embedding performance.

## Implementation and scope

The observer lives in the existing
[preparation owner](../../crates/fastxslt/src/runtime/prepared_input_experiment.rs).
Its parameters, clocks and observation type are compiled only for tests.
Ordinary production parsing, construction, controls and diagnostics remain
unchanged. There is no timing export, global subscriber, new dependency or
alternative semantic implementation.

The [private phase tests](../../crates/fastxslt/src/runtime/prepared_input_experiment/preparation_phase_tests.rs)
compare the observed and ordinary calls through that same owner. They check node
kinds, names, prefixes, values, relationships, namespaces, document order,
provenance, node count and accounted capacities. Separate constructions retain
distinct origins. XML-event and XDM-node charges match, exact ceilings succeed,
one-less ceilings fail identically, and cancellation in either phase preserves
the same failure. Malformed XML and missing-resource diagnostics also match.
Both paths produce exact serialized results.

Unlike the earlier separate-phase microprobe, this probe does not preconstruct
a vector of parsed trees outside the XDM timing window. Each sample parses and
constructs one document in sequence with one control. The timing boundaries are:

- **XML:** controlled parsing, owned event/name/value creation, namespace and
  well-formedness checks, and successful diagnostic projection. This is not
  isolated `quick-xml` tokenizer time.
- **XDM:** controlled conversion of those owned events into the safe tree,
  including the document Arc allocation and parser-state release during
  conversion. No alternative representation is used.
- **Complete preparation:** snapshot lookup through returned document/capacity.
  This includes the existing parsed-capacity walk between the two phase clocks.
  Source admission, compilation, execution, result validation, document release
  and prepared-map insertion/sealing are outside this clock.

The paired ordinary call omits the internal phase clocks but retains the same
outer preparation clock. Test-only code shape and clock overhead are not assumed
to equal production adapter costs. Phase medians are not an additive breakdown
of the complete median.

## Reproduction and method

```text
cargo test -p fastxslt --all-features phase_observation -- --nocapture
cargo test -p fastxslt --release --features workbench measure_shared_preparation_phases -- --ignored --nocapture
```

Three fresh processes ran after workspace verification and the WASM check
completed. Each compiles the unchanged stylesheet once and admits source bytes
before timing. Every fixture gets 32 warmups for each path and 1,001 paired
samples, alternating ordinary/observed order. Fixture order is fixed; machine
activity, frequency, affinity and cross-machine behavior are not controlled.
Process test windows were approximately 2.1-2.2 seconds, including all fixtures,
execution/serialization validation and report bookkeeping. No convergence or
competitive publication gate is claimed.

The synthetic inputs contain 5, 50 or 500 empty `order-item` elements with
`price="1" qty="1"`, under `order`; exact expected values are `5.00`, `50.00`
and `500.00`. They are not upstream conformance inputs, a representative
consumer distribution or new corpus passes. The pinned source remains byte
unchanged. All 24,024 timed outcomes compare exactly.

## Observations

All phase values are per-process medians in microseconds.

| Input | Bytes | Run | XML | XDM | Instrumented preparation | Ordinary preparation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Pinned `for03.xml` | 216 | 1 | 5.7 | 2.0 | 7.7 | 7.7 |
| Pinned `for03.xml` | 216 | 2 | 5.7 | 1.9 | 7.7 | 7.6 |
| Pinned `for03.xml` | 216 | 3 | 8.0 | 2.6 | 10.9 | 10.7 |
| Synthetic 5 | 170 | 1 | 5.0 | 1.6 | 6.7 | 6.7 |
| Synthetic 5 | 170 | 2 | 5.1 | 1.6 | 6.8 | 6.8 |
| Synthetic 5 | 170 | 3 | 8.0 | 2.4 | 10.7 | 10.4 |
| Synthetic 50 | 1,565 | 1 | 60.3 | 33.3 | 96.0 | 97.5 |
| Synthetic 50 | 1,565 | 2 | 63.2 | 43.5 | 110.1 | 108.8 |
| Synthetic 50 | 1,565 | 3 | 69.5 | 34.9 | 104.8 | 105.5 |
| Synthetic 500 | 15,515 | 1 | 497.3 | 126.7 | 637.8 | 644.4 |
| Synthetic 500 | 15,515 | 2 | 483.1 | 132.1 | 633.7 | 622.8 |
| Synthetic 500 | 15,515 | 3 | 486.3 | 147.2 | 646.5 | 646.3 |

Observed and ordinary preparation medians differ by less than 3% in these
cells, in either direction. That is a local overhead sanity check, not proof
that clock overhead is zero. The tiny source still varies substantially across
processes. These timings cannot be subtracted from historical adapter or
lifecycle medians to manufacture a boundary-cost number.

## Interpretation and next work

The XML boundary is the larger measured preparation phase for these sources;
XDM construction is also material. A compact XDM representation may reduce
retained memory or construction cost, but these data do not establish such a
win, nor identify which XML suboperation is expensive. Parser replacement and
staging migration are not justified by phase totals alone.

Next compare native/isolated creation costs with matched lifecycle scopes and
measure live records versus vector slack across deep, attribute-heavy,
text-heavy, namespace-heavy and low-repetition sources. Keep that anatomy track
separate from adapter transfer/copy attribution. Representative 5,000-document
ingestion, allocation/copy counts, peak coexistence and tail measurements remain
open.

## Cohesion and validation

The 269-line private child owns phase-attribution tests and timing only. The
1,130-line preparation parent retains lifecycle composition and the small
test-only observer at the actual XML/XDM seam. It remains cohesive at this
checkpoint; no semantic extraction or public provider is needed for observation.
The named preparation subdirectory has two Rust children, and the dense runtime
root receives no new direct source file. Dependencies remain one-way into the
existing owners; neither test child depends on the other.

Full verification passes: formatting, strict all-feature Clippy, 1,510 core tests
with 38 manual probes ignored, other workspace tests, documentation, unsafe
surface, Markdown links and conformance inventories. The official
`wasm32-unknown-unknown` adapter check passes. Existing build-cache hard-link and
narrow-feature unused-code warnings were not suppressed. Corpus bytes,
selection and compatibility counts are unchanged.
