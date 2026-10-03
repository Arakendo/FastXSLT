# Full transform allocation peaks

Date: 2026-10-03. Scope: AR-0027's private node-capacity candidates through a
single-thread Rust source-admission, preparation, execution, serialization and
release lifecycle. Compilation is retained outside these scopes.

Pre-sizing reduces peak requested allocation on the admitted full-transform
shapes, but the benefit depends on the result workload. Full-copy peak falls
19.0% on wide and 47.7% on attribute-heavy sources, versus only 1.9% on the
text-heavy source. These are allocation observations, not throughput, process
memory or adoption claims. This initial matrix exposed a default-debug-stack
failure. The [subsequent repair and full six-copy replay](ar-0027-stack-safe-serialization-and-allocation-replay-2026-10-03.md)
resolve that blocker; the tables below remain the original pre-repair observations.

## Measurement scope

The [private probe](../../crates/fastxslt/src/runtime/prepared_input_experiment/capacity_transform_peak_tests.rs)
uses the existing six-shape anatomy fixtures and three construction policies:
ordinary growth, test-only freeze and test-only pre-sizing. Each workload has
fresh complete prefix scopes ending with:

- an admitted snapshot and prepared document;
- snapshot, document and semantic result;
- snapshot, document, result and serialized string;
- complete execution and release of all scope-owned state.

Caller fixture strings, compiled programs, independently specified element-count
answers and reference-copy expectations remain outside the measured scope.
Source admission's owned byte copy, XML parsing, XDM construction, invocation
work and retained result buffers are inside it. Measurements stay on one thread.

Counters are not nested: allocation-counter 0.8.1 merges nested maxima additively,
which would not establish simultaneous peak retention. Each fresh prefix has
its own actual high-water observation. Released scopes include the entire
lifecycle and prove zero net tracked allocations/bytes. Earlier prefix values
describe still-owned state, which is dropped outside that prefix's observation.
These requested-byte metrics are neither RSS nor allocator/OS reclamation.

The count workload returns an independently specified element count. The copy
workload copies the full source document; its exact output is compared with the
unchanged growth-tree reference, serialized after source retirement. That is a
differential oracle, not independent standards authority. Each measurement run
has 132 prefix scopes, 66 exact serialized comparisons and 33 complete release
controls. Three fresh release processes reproduce the allocation records exactly.

## Full lifecycle peak requested bytes

All values below are bytes. Reduction compares the pre-sized peak with ordinary
growth for the same workload. Deep copy is not a completed matrix cell.

| Shape | Workload | Growth peak | Freeze peak | Pre-sized peak | Pre-sized reduction |
| --- | --- | ---: | ---: | ---: | ---: |
| Wide | Count | 2,395,411 | 2,395,411 | 1,845,483 | 23.0% |
| Wide | Full copy | 2,395,411 | 2,395,411 | 1,940,863 | 19.0% |
| Deep | Count | 349,061 | 349,061 | 226,709 | 35.1% |
| Attribute-heavy | Count | 855,763 | 855,763 | 447,459 | 47.7% |
| Attribute-heavy | Full copy | 855,763 | 855,763 | 447,459 | 47.7% |
| Text-heavy | Count | 661,767 | 661,767 | 598,831 | 9.5% |
| Text-heavy | Full copy | 1,683,103 | 1,651,855 | 1,651,855 | 1.9% |
| Namespace-heavy | Count | 252,667 | 252,667 | 190,206 | 24.7% |
| Namespace-heavy | Full copy | 279,385 | 252,667 | 248,137 | 11.2% |
| Low-repetition | Count | 1,207,091 | 1,207,091 | 942,337 | 21.9% |
| Low-repetition | Full copy | 1,207,091 | 1,207,091 | 993,679 | 17.7% |

Preparation remains the whole-lifecycle high-water for several count and copy
cells. Text-heavy copying instead raises the observed peak after result and
serialization work. Its retained growth-prefix bytes are 595,177 prepared,
894,906 with the semantic result and 1,420,346 with the serialized string. The
pre-sized equivalents are 563,929, 863,658 and 1,389,098: the same 31,248-byte
prepared-storage saving persists, but is a small share of the larger lifecycle.
Thus a preparation-only percentage does not predict the end-to-end percentage.

## Deep copy reference limitation

The unchanged 256-deep growth-tree copy reference terminates the default debug
test-thread process with `0xc00000fd / STATUS_STACK_OVERFLOW`. A separate
sacrificial reproducer confirms this independently of the allocation matrix.
The same reproducer passes in the optimized release build and returns the exact
256 nested elements plus leaf text. No stack-size override was introduced.

The matrix excludes deep copy in both build modes so the release results do not
quietly erase the debug failure. Deep count still executes. The crash test is
explicitly ignored, names the open baseline limitation and is not counted as a
pass. A [subsequent phase probe and extraction checkpoint](ar-0027-serialization-stack-localization-2026-10-03.md)
now confirms that copying and source retirement complete before the overflow
during serialization. The subsequent repair linked above resolves the observed
blocker and restores the deep-copy cell; this historical observation does not
attribute the crash to a capacity candidate or establish a production depth
guarantee. The ignored-crash commands below describe the pre-repair state.

```text
cargo test -p fastxslt --all-features capacity_full_transform_allocation_scopes -- --nocapture
cargo test -p fastxslt --release --all-features measure_capacity_full_transform_allocations -- --ignored --nocapture
cargo test -p fastxslt --all-features capacity_full_transform_deep_copy_reference -- --ignored --nocapture
cargo test -p fastxslt --release --all-features capacity_full_transform_deep_copy_reference -- --ignored --nocapture
```

The third command crashes on the observed default debug harness. Use a
sacrificial process; do not treat running all ignored probes together as a gate.

## Validation and remaining work

Full verification passes with 1,535 core tests and 49 ignored probes. Formatting,
strict Clippy, workspace tests, documentation, local links, conformance integrity
and unsafe-surface checks pass. The official WASM build check passes with
filesystem hard-link fallback warnings. The allocation matrix was not run in
WASM and its host-width observations must not be projected there.

The original 183-line probe (206 lines after phase markers) is a private
allocation-observation child of the existing
225-line lifecycle owner. It adds no semantic implementation and depends on
existing source, runtime and serializer owners. The preparation-test directory
now has ten direct files; the new file is named for its measured responsibility.
No layout refactor, public API, dependency, unsafe surface or host policy changes.

AR-0027 remains Incubating. Production capacity construction and DTD denial are
unchanged, and OASIS remains 2,485 / 3,173 (78.32%). The deep-copy blocker has
since been resolved and remeasured in the linked follow-up. Continue to
extend failure/reuse controls and representative adapter measurements, and review
checkpoint publication before adopting either capacity candidate. No break-even
reuse count or general speedup follows from these allocation-only observations.
