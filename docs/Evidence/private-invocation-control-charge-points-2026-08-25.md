# Private Invocation-Control Charge Points

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Revision under test | Working tree after `ad0e688` |
| Scope | Private AR-0010 cooperative cancellation and deterministic work accounting experiment |
| Informs | AR-0010, AR-0004, and M1 vertical-slice planning |
| Public guarantee | None |

## Experiment shape

Each transform request carries a private cancellation token backed by an atomic
flag. The host side may retain a clone and signal it without mutating engine
semantic state. When execution begins, the request receives fresh counters from
the transform-set policy; counters are not shared between sibling invocations.

Every charge first observes cancellation and then attempts to debit one named
work domain. Cancellation and budget exhaustion become separate structured
operation failures with request identity and work-domain identity:

```text
FXCT0001 / Cancelled -> host cancellation observed at a charge point
FXCT0002 / Limit     -> one named work domain cannot admit its next charge
```

The identifiers and private types remain experimental.

## Implemented inventory

| Domain | Unit charged | Current charge/check point | Separate structural control |
| --- | --- | --- | --- |
| XML event | One non-EOF parser event returned | Immediately after the parser returns an event and before FastXSLT handles it | Parser event count and XML depth |
| XDM node | One document, element, attribute, text, comment, or processing-instruction node actually allocated | Before each node enters the owned document | None beyond the work limit yet |
| XDM string-value node | One node visited while computing string value | On entry to each recursive node visit | None yet |
| XPath node visit | One candidate child inspected | Inside the child-axis loop, before kind/name tests | Result cardinality remains a separate semantic concern |
| XSLT instruction | One instruction entered | Before dispatching each literal-element, text, or value-of instruction | None yet |
| Result node | One element or non-empty text node reserved for the semantic result | Before constructing the result node or its descendants | Independent of serialized representation |
| Result text byte | UTF-8 bytes about to be retained in semantic result text | Before creating or extending a result text node | Does not cover temporary XDM string-value allocation |
| Serialized byte | UTF-8 bytes about to be appended | Before each in-memory serializer write | Separate maximum serialized-result bytes |

The XPath unit deliberately reflects data-dependent scanning. A path step over
800,000 candidate children costs 800,000 visits rather than one expression
unit. This is evidence for the current child-axis evaluator only; it does not
define weights for predicates, functions, regular expressions, sorting, joins,
keys, or future optimized operations.

## Results

Forty-one ordinary tests pass and one manual release-mode measurement test is
ignored by default. Focused cases prove:

- a boundary-sized charge succeeds and the next charge reports its domain,
  configured limit, prior consumption, and attempted debit;
- a host cancellation signalled after request admission is classified as
  cancellation rather than budget exhaustion and retains request identity;
- XML parsing, XDM construction, XPath child scanning, XDM string-value
  traversal, XSLT instruction dispatch, result-node creation, result text
  retention, and serialization each fail through their own exhausted work
  domain;
- result text counts UTF-8 bytes: one node containing `🚀` fits an exact
  four-byte result-text budget, while the next byte fails before serialization;
  and
- the existing serialized-output limit remains a separate failure without a
  work-domain label.

The golden source path has this exact current charge profile:

| Domain | Consumed units |
| --- | ---: |
| XML event | 10 |
| XDM node | 6 |
| XSLT instruction | 4 |
| XPath node visit | 4 |
| XDM string-value node | 2 |
| Result node | 2 |
| Result text byte | 16 |
| Serialized byte | 35 |

This fixture-specific conservation assertion catches an accidentally removed,
duplicated, or reassigned charge. It is not a budget recommendation.

## Observation-gap inventory

| Domain | Maximum gap in the current semantic unit | Work hidden inside one gap |
| --- | --- | --- |
| XML event | One returned non-EOF event | One `quick-xml` read/decode plus FastXSLT handling between returned events; event byte size is bounded only by admitted input policy |
| XDM node | One reserved node | Cloning the owned event fields and linking that node; parser/event construction happened earlier |
| XDM string-value node | One visited node | Node dispatch and at most one borrowed fragment callback; fragment append has its own byte charge |
| XPath node visit | One candidate child | Kind/name checks for that candidate |
| XSLT instruction | One entered instruction | Instruction-specific work, which uses narrower XPath, XDM, result, and serializer checks where implemented |
| Result node | One reserved semantic node | Element-name clone or text-node creation; descendant construction has its own checks |
| Result text byte | One borrowed fragment's UTF-8 length | One `String` append; cancellation is not observed per byte inside that append |
| Serialized byte | One serializer append chunk's UTF-8 length | Capacity growth and copy for that chunk; text escaping currently emits per character or escape sequence |

These are maximum gaps in named semantic units, not time. A single dependency
call, allocation, or large fragment can take variable wall time, so no deadline
or maximum cancellation latency follows.

## Local accounting-cost probe

The final ignored probe implementation was run three times with:

```text
cargo test --release --workspace --all-features measures_unexhausted_charge_cost -- --ignored --nocapture
```

Environment: Rust/Cargo 1.95.0, `x86_64-pc-windows-msvc`, LLVM 22.1.2,
`AMD64 Family 25 Model 97 Stepping 2`, 16 reported logical processors. Each run
used seven samples of 10,000,000 successful `XPathNodeVisit` charges.

| Run | Black-box loop median | Successful charge median | Difference |
| --- | ---: | ---: | ---: |
| 1 | 0.205 ns/iteration | 1.249 ns/charge | 1.044 ns |
| 2 | 0.205 ns/iteration | 1.241 ns/charge | 1.036 ns |
| 3 | 0.207 ns/iteration | 1.215 ns/charge | 1.008 ns |

The probe measures one uncontended atomic cancellation read, domain lookup,
counter comparison, and decrement in a tight optimized loop. The subtraction is
descriptive, not a statistically isolated causal cost. It does not measure a
transform, cache behavior, contention, cancellation signalling, pipeline
slowdown, tail latency, or an ASP.NET boundary and cannot justify defaults.

## Phase-specific cancellation injection

A test-only fault injector now signals the same atomic cancellation token when
execution reaches a selected real charge point. It can first allow a specified
number of matching charges to succeed, which distinguishes cancellation after
partial work from a token cancelled before execution begins.

The complete private transform-set path injects cancellation after earlier work
in each of the eight implemented domains. Every case returns `FXCT0001`, the
selected work domain, and the logical request identity. A separate two-request
case lets one sibling complete before a later request cancels during
serialization; the current reference operation returns the cancellation and no
partial `ResultSet`.

That all-result-or-failure behavior is evidence about the private reference
executor only. It does not decide whether a future public batch API fails fast,
collects per-request failures, or offers an explicitly named partial-results
surface.

## Observation gaps and limitations

Cancellation is cooperative. “Checked once per XML event” means after the
parser dependency returns that event. It does not interrupt work inside one
`read_event`, name/attribute decoding operation, allocation, atomic operation,
or operating-system/dependency call. Equivalent intra-operation gaps exist in
the other layers. Maximum wall-clock observation latency is therefore neither
known nor guaranteed.

The deterministic fault injector is test machinery, not a host timing model.
It proves behavior when a charge point observes cancellation; it does not
measure how long a host signal takes to reach that point.

This experiment has no deadline, dispatcher, thread pool, async task, panic
containment, process isolation, calibrated default, aggregate cross-domain
budget, allocation meter, benchmark, public API, or ASP.NET adapter. The
messages and diagnostic growth do not yet have their own work domains. Dynamic
XDM string values now visit borrowed fragments in semantic order and write them
directly through result construction, avoiding an aggregate temporary string.
The source fragment strings remain retained by the owned XDM, and this does not
bound unrelated runtime temporaries or establish streaming behavior.

The work limits use independent counters so evidence remains attributable.
Whether a future host receives individual controls, a composed total budget, or
both requires representative workloads and overhead measurement.
