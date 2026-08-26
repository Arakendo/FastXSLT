# Private Invocation-Control Charge Points

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Revision under test | Working tree after `11f9ae5` |
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
| Serialized byte | UTF-8 bytes about to be appended | Before each in-memory serializer write | Separate maximum serialized-result bytes |

The XPath unit deliberately reflects data-dependent scanning. A path step over
800,000 candidate children costs 800,000 visits rather than one expression
unit. This is evidence for the current child-axis evaluator only; it does not
define weights for predicates, functions, regular expressions, sorting, joins,
keys, or future optimized operations.

## Results

Thirty-eight tests pass. Focused cases prove:

- a boundary-sized charge succeeds and the next charge reports its domain,
  configured limit, prior consumption, and attempted debit;
- a host cancellation signalled after request admission is classified as
  cancellation rather than budget exhaustion and retains request identity;
- XML parsing, XDM construction, XPath child scanning, XDM string-value
  traversal, XSLT instruction dispatch, and serialization each fail through
  their own exhausted work domain; and
- the existing serialized-output limit remains a separate failure without a
  work-domain label.

## Phase-specific cancellation injection

A test-only fault injector now signals the same atomic cancellation token when
execution reaches a selected real charge point. It can first allow a specified
number of matching charges to succeed, which distinguishes cancellation after
partial work from a token cancelled before execution begins.

The complete private transform-set path injects cancellation after earlier work
in each of the six implemented domains. Every case returns `FXCT0001`, the
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
current result-construction nodes/bytes, messages, and diagnostic growth do not
yet have their own work domains.

The work limits use independent counters so evidence remains attributable.
Whether a future host receives individual controls, a composed total budget, or
both requires representative workloads and overhead measurement.
