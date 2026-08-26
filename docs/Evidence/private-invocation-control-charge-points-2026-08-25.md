# Private Invocation-Control Charge Points

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Revision under test | Working tree after `ea105f3` |
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

Thirty tests pass. Focused cases prove:

- a boundary-sized charge succeeds and the next charge reports its domain,
  configured limit, prior consumption, and attempted debit;
- a host cancellation signalled after request admission is classified as
  cancellation rather than budget exhaustion and retains request identity;
- XML parsing, XDM construction, XPath child scanning, XDM string-value
  traversal, XSLT instruction dispatch, and serialization each fail through
  their own exhausted work domain; and
- the existing serialized-output limit remains a separate failure without a
  work-domain label.

## Observation gaps and limitations

Cancellation is cooperative. “Checked once per XML event” means after the
parser dependency returns that event. It does not interrupt work inside one
`read_event`, name/attribute decoding operation, allocation, atomic operation,
or operating-system/dependency call. Equivalent intra-operation gaps exist in
the other layers. Maximum wall-clock observation latency is therefore neither
known nor guaranteed.

This experiment has no deadline, dispatcher, thread pool, async task, panic
containment, process isolation, calibrated default, aggregate cross-domain
budget, allocation meter, benchmark, public API, or ASP.NET adapter. The
cancellation case is signalled before execution; cancellation injected during
each execution phase and partial-result policy remain required follow-up.

The work limits use independent counters so evidence remains attributable.
Whether a future host receives individual controls, a composed total budget, or
both requires representative workloads and overhead measurement.
