# AR 0027 Bounded Capacity Checkpoints

| Field | Value |
| --- | --- |
| Date | 2026-10-02 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Prior evidence | [Unchecked capacity intervals](ar-0027-capacity-cancellation-gaps-2026-10-02.md) |
| Scope | Test-only uncharged cancellation polls during construction and capacity changes |
| Disposition | Boundary controls pass; publication rule and production adoption remain open |

The private checkpoint candidate observes scan cancellation within 256 event
visits and rejects cancellation before/after reservation or shrinking without
returning a document. Work charges and charge-indexed faults remain unchanged
in the differential controls. The candidate is not enabled in production and
does not establish a hard cancellation deadline.

## Candidate and completion rule

The [XDM constructor](../../crates/fastxslt/src/xdm/owned_tree_experiment.rs)
polls before every 256-event scan chunk, at scan completion, and before/after
the selected resize. A test-only invocation-control method reads the existing
cancellation token without consuming units, resetting observations or advancing
the charge-indexed fault injector. Ordinary node admission still uses the
existing charges. The production span scan retains its original iterator path;
the experimental loop, polls, observer and constructor entry point are test-only.

The chunk size is an experimental implementation choice, not a host setting or
supported numerical promise. A visit bound is not a wall-clock bound: thread
scheduling, cleanup and allocator calls can delay return. Polling around an
allocator call does not interrupt that call.

The freeze candidate performs a final cancellation poll after shrinking:
a token observed there rejects the document even though semantic construction
has completed. The unpolled freeze reference instead returns success when the
same observer signals at that boundary. This difference is explicit, not claimed
as universal cancellation parity. The final poll is not atomic with constructor
return or caller publication; an unobserved later signal cannot retroactively
revoke a returned immutable document. Calling this a publication fence would
overstate the implementation. A supported completion rule still requires a
decision before adoption; no ABI, host cancellation contract or production
behavior is changed here.

The [2026-10-03 handoff controls](ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#candidate-publication-boundary-review-on-2026-10-03)
exercise failed replacement, old/new owner overlap, and cancellation after return.
They review the private candidate's behavior without selecting atomic publication
or a production completion rule.

## Differential and boundary evidence

The [225-line checkpoint test owner](../../crates/fastxslt/src/runtime/prepared_input_experiment/capacity_checkpoint_tests.rs)
has three ordinary controls:

- Ten scan controls signal at visits 1, 255, 256, 257 and the final 3,074th event,
  in growth and pre-sizing. Fewer than 256 additional events are visited after
  each signal; cancellation returns with zero XDM-node units consumed.
- Twenty-four resize controls cover six shapes, reservation/shrinking and
  signalling before/after resizing. Before-resize cancellation prevents the
  after-resize checkpoint; after-resize cancellation also returns failure.
  Consumption is one document node for reservation and all nodes for shrinking.
- Two hundred sixteen differential pairs cover six shapes, three constructors,
  exact/one-less/zero node ceilings, and absent/first/second/last-node charge
  faults. Successful capacity anatomy and string values match; failures and
  consumed XDM-node work match. Extra polls do not shift charge-indexed faults.

These fixtures do not prove every malformed-event/cancellation race, complete
allocation reclamation under asynchronous faults, or adapter publication parity.
The earlier unpolled controls remain separate references.

## Rotated construction observations

Run three fresh release processes without allocation instrumentation:

```text
cargo test -p fastxslt --release --features workbench measure_capacity_checkpoint_overhead -- --ignored --nocapture --test-threads=1
```

Each process uses the six anatomy shapes and the 96,002-node stress source.
Six lanes pair growth, pre-sizing and freezing with their polled equivalents.
Lane order rotates every round; three rounds warm up, followed by 31 samples
per cell. Parsing occurs before timing; construction consumes the parsed source,
and document release occurs afterward. No observer callback is supplied in the
timing lanes. Node count and consumed node units are validated on every sample.
This is XDM construction timing, not complete ingestion, host performance or
wall-clock cancellation latency. Process windows are 28.08, 48.08 and 35.92 s.

The table gives ranges of the three paired process-median latency ratios.
Below one means the polled lane measured faster, not that polling improves
construction. These ranges are observations, not confidence intervals.

| Shape | Growth polls/reference | Presizing polls/reference | Freezing polls/reference |
| --- | ---: | ---: | ---: |
| wide | 0.998-1.077 | 0.977-1.064 | 1.019-1.145 |
| deep | 0.865-0.954 | 0.946-1.086 | 0.972-1.159 |
| attribute-heavy | 0.782-1.119 | 0.825-1.008 | 0.751-1.371 |
| text-heavy | 0.992-1.091 | 0.755-1.000 | 0.862-0.939 |
| namespace-heavy | 0.975-1.100 | 0.982-1.088 | 0.847-1.008 |
| low-repetition | 0.820-1.290 | 0.900-1.007 | 0.944-1.150 |
| wide-32000 | 0.956-1.129 | 1.004-1.127 | 1.047-1.093 |

Variation is too large to identify a precise polling cost. For example,
low-repetition growth changes from 0.820 to 1.290 times its paired reference.
Stress pre-sizing is 1.004-1.127 times reference; stress freezing is slower
in all three windows. Do not call the polls free or infer a general regression
from these short variable runs. The stress growth/polled-growth medians are
48.669/49.328, 62.913/71.014 and 57.167/54.644 ms. Its p95 values range from
79.171 to 325.354 ms across those two lanes, further exposing tail variability.
Longer repeated windows and complete-source/adapter measurements are required
before accepting an overhead claim or a production checkpoint policy.

## Validation and remaining work

Full workspace verification passes with 1,525 core tests and 47 ignored probes,
including strict Clippy, formatting, docs, links, conformance and unsafe-surface
checks. The official WASM build passes with filesystem hard-link fallback
warnings. Candidate timing and controls are native test builds, not WASM trials.

The 1,310-line XDM owner retains constructor-local experiment hooks at this
checkpoint: sharing the event-to-tree state machine avoids a second builder.
The test child owns polling faults and overhead measurements, depends on the
fixture parent and existing semantic owners, and does not access sibling-test
internals. Eight direct preparation-test files remain below directory-density
review pressure. There is no new public API, authority, registry metric,
scheduler or unsafe surface. Reassess extraction if the candidate is selected
or the constructor experiments acquire a separate stable responsibility.

Complete adapter retention, broader semantic parity, exact copied bytes, full
transform peak and the supported completion rule remain open. Corpus counts
are unchanged; capacity and checkpoint candidates remain private.
