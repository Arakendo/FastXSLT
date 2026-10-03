# Stack safe result serialization and allocation replay

- Date: 2026-10-03
- Status: Localized blocker resolved; capacity adoption remains open
- Related review: [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md)
- Previous checkpoint: [Serialization failure localization](ar-0027-serialization-stack-localization-2026-10-03.md)

The unchanged 256-deep ordinary growth-tree copy now passes on the default
debug test-thread stack, including source retirement before serialization and
exact output. The recursive node writer is replaced with safe borrowed
ancestor cursors. This fixes the observed serialization crash without changing
the copy evaluator, prepared layout, host policy or harness stack.

## Traversal and conservation

The private writer retains one cursor per active element ancestor, rather than
one queued event per descendant. Each cursor borrows the semantic result's name
and children and owns its namespace restoration frame and selected prefix.
Children are visited in the same order; no result node or payload is copied.
The cursor vector is temporary and drops before serialization returns.

Depth drives scratch retention, not source breadth. Cursors are pushed only
after the existing start-tag writes succeed through the budgeted sink, so
output ceilings and charged output progress still stop descent. Scratch is not
an allocator-exact memory quota or a new public limit. Namespace state is
restored in reverse order on normal closing, opening failure, nested failure
and closing failure. All existing output writes and their charge ordering remain
unchanged.

The unchanged recursive element/child traversal remains a private test-only
shallow oracle. It shares the unchanged leaf-emission and attribute helpers;
it is not a second production backend. Differential controls cover 108
combinations of XML/HTML/XHTML modes, namespace shapes, indentation and
empty-tag conventions, including CDATA, raw text, character maps, content-type
insertion, attributes, comments and processing instructions.

A second control compares exact failure, already-written byte prefix and
serialized-byte consumption at every output-byte ceiling and work-byte ceiling
from zero through complete length. Deterministic cancellation is compared at
every possible charge index, including after completion. A subsequent probe
verifies that the root's namespace binding has been removed after every attempt.
These are writer controls, not new conformance cases.

The default-debug deep reference is no longer ignored. The allocation matrix
now includes six count and six copy shapes, using an independently specified
256-element nested expected string for deep copy. It checks 72 complete
serialized outcomes across 144 fresh prefix scopes and 36 full-release scopes
per run. The scope and non-nesting rules are unchanged from the
[original allocation record](ar-0027-full-transform-allocation-peaks-2026-10-03.md).

## Repeated release allocation observations

Three fresh release processes reproduce all 144 allocation records identically.
Every full-release scope ends with zero tracked live allocations and bytes.
These are requested allocator bytes on this host, not RSS, CLR memory,
throughput, WASM-width estimates or a deployment ceiling.

| Shape | Workload | Growth peak bytes | Freeze peak bytes | Pre-sized peak bytes | Pre-sized reduction |
| --- | --- | ---: | ---: | ---: | ---: |
| wide | count | 2,395,411 | 2,395,411 | 1,845,483 | 23.0% |
| wide | copy | 2,395,411 | 2,395,411 | 1,941,663 | 18.9% |
| deep | count | 349,061 | 349,061 | 226,709 | 35.1% |
| deep | copy | 349,061 | 349,061 | 277,893 | 20.4% |
| attribute-heavy | count | 855,763 | 855,763 | 447,459 | 47.7% |
| attribute-heavy | copy | 855,763 | 855,763 | 447,459 | 47.7% |
| text-heavy | count | 661,767 | 661,767 | 598,831 | 9.5% |
| text-heavy | copy | 1,683,903 | 1,652,655 | 1,652,655 | 1.9% |
| namespace-heavy | count | 252,667 | 252,667 | 190,206 | 24.7% |
| namespace-heavy | copy | 280,185 | 252,667 | 248,937 | 11.2% |
| low-repetition | count | 1,207,091 | 1,207,091 | 942,337 | 21.9% |
| low-repetition | copy | 1,207,091 | 1,207,091 | 994,479 | 17.6% |

The newly admitted deep-copy cell reduces full-lifecycle peak by 20.4% with
pre-sizing, versus 35.1% for deep count. The result and serializer scratch consume
part of the saved preparation headroom. The advantage remains workload-dependent.

Some copy peaks rise by 800 requested bytes compared with the previous record:
the explicit cursor vector adds transient heap storage where recursive frames
previously used thread-stack storage. For example, text-heavy growth rises from
1,683,103 to 1,683,903 bytes and namespace-heavy growth from 279,385 to 280,185.
This is a visible repair cost, not a prepared-capacity regression or an OS
memory comparison. Prepared/result/serialized retained ownership is unchanged;
the cursor allocation is released before the serialized prefix is retained.

## Validation and remaining work

```text
cargo test -p fastxslt --all-features iterative_writer -- --nocapture
cargo test -p fastxslt --all-features capacity_full_transform_deep_copy_reference -- --nocapture
cargo test -p fastxslt --release --all-features measure_capacity_full_transform_allocations -- --ignored --nocapture
pwsh -NoProfile -File scripts/verify.ps1
cargo check -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

Full gates pass with 1,538 core tests and 48 ignored probes, plus all adapter
tests, formatting, strict Clippy, docs, links, corpus integrity and the reviewed
unsafe surface. The official WASM target builds. Cache hard-link fallback
warnings remain. No host throughput benchmark was rerun and no speed claim
follows from allocation replay.

The production node writer is 403 lines, with a private 149-line recursive
oracle and focused child tests. Dependencies stay within the serialization
owner; no public API, crate, dependency, acquisition authority or unsafe code
is added. The 2,560-line serializer parent still has documented cohesion
pressure.

This closes the observed 256-deep XML-output blocker, not every recursive
operation in the engine. Text-method traversal, validation, copying and result
disposal are separate owners or paths; no arbitrary-depth or all-method
stack-safety guarantee is inferred.

AR-0027 remains Incubating. Production node-vector construction and DTD denial
are unchanged. OASIS stays 2,485 / 3,173 (78.32%). Next work is broader reuse,
representative native/isolated ingestion attribution and review of capacity
checkpoint/publication behavior before adopting a candidate.
