# AR-0023: Bounded XSLT Message Observation and Delivery

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-09-23 |
| Last reviewed | 2026-09-23 |
| Scope | `xsl:message` semantics, invocation ownership, accounting, and host delivery |
| Trigger | Twenty-seven OASIS XSLT 1.0 cases stop at the unsupported `xsl:message` instruction while the SDD and AR-0010 already require messages to be invocation-owned and bounded |
| Related ADRs | ADR-0005, ADR-0006, ADR-0019 |
| Related reviews | AR-0004, AR-0010, AR-0019, AR-0021 |
| Related evidence | The hash-verified OASIS XSLT 1.0 measurement and `unsupported/FXST1006/unsupported XSLT instruction: xsl:message` frontier |

## Architectural question

How should FastXSLT execute, bound, retain, and deliver `xsl:message`
observations—including terminating messages—without turning messages into
ambient logging, conflating them with diagnostics, or selecting a premature
public callback and transport contract?

## Trigger and evidence

The local OASIS XSLT 1.0 sweep reaches 27 ordinary `xsl:message` instructions.
The cases exercise default and explicit non-termination, termination, invalid
attributes and lexical values, nested construction, imported instructions, and
message content containing XML fragments. The current compiler rejects every
case at the instruction boundary, so the corpus does not yet establish runtime
message semantics.

This is not only a legacy feature. The SDD already says messages are invocation
state and that a transformation result must be able to return messages without
a hidden side channel. AR-0010 explicitly leaves message accounting unfinished.
ADR-0005 requires each transform-set member to retain independent identity and
outcomes, while ADR-0019 requires isolated incremental transport to preserve
independent message semantics.

No current consumer establishes whether the eventual Rust/.NET surface should
return a retained collection, stream callbacks, expose asynchronous iteration,
or combine mechanisms. No measurements yet establish realistic count, byte,
construction, or transport pressure.

## Ownership and constraints

- A message is an ordered observation produced by one invocation. It is not
  compiled state, prepared input, a sibling-visible resource, or ambient
  process logging.
- FastXSLT owns XSLT message construction, termination semantics, ordering,
  work charging, bounded retention, and structured failure identity.
- The host owns logging, persistence, redaction, publication, alerting, and any
  decision to expose authored message content outside its trust boundary.
- Message content must not mutate the principal semantic result. A terminating
  message yields no successful partial transformation result.
- A message is distinct from an engine diagnostic. Authored content and a
  standard error caused by `terminate="yes"` may coexist in one invocation
  outcome without being collapsed into one display string.
- Count, constructed content, retained UTF-8-equivalent bytes, and transport
  bytes are separate pressure dimensions. Limits must be checked before
  unbounded retention.
- Cancellation and work budgets remain observable while constructing and
  delivering messages. A message callback cannot become an unbounded or
  non-cooperating dependency hidden inside semantic execution.
- Direct Rust, native .NET, isolated single-member, and isolated transform-set
  paths must preserve semantic ordering and diagnostic identity before they are
  presented as equivalent.
- An isolated worker loss may leave admitted but unobserved messages ambiguous;
  sequence position alone must not manufacture delivery certainty.
- Message content may contain sensitive data intentionally selected by a
  stylesheet. Engine tracing and failure envelopes must not duplicate it
  automatically.

## Alternatives

### A. Invocation-owned retained messages in an outcome envelope

Construct bounded semantic messages in order and return them alongside either
the successful result or the operation failure. This gives direct execution a
simple reference oracle and preserves terminating-message observations.

The cost is retained memory until the outcome is consumed. A collection-only
surface may also delay observation and increase isolated response frames.

### B. Synchronous or asynchronous host callback

Deliver each bounded message as it occurs. This can reduce retained memory and
latency, but introduces reentrancy, host blocking, exception/failure, thread
affinity, cancellation, and cross-process backpressure questions. It is too
large a contract to select from corpus cases alone.

### C. Private collection oracle plus later incremental delivery

Use a bounded invocation-owned collection as the semantic reference. Compare a
private incremental delivery experiment against it, much as aggregate isolated
transport remains the oracle for incremental result delivery. Stabilize only
behavior that survives direct/native/isolated parity and abandonment tests.

This is the leading experiment. It separates semantic admission from the public
host interface and supplies a differential oracle if delivery later becomes
incremental.

### D. Emit messages through ambient logging

This loses per-request ownership, can leak sensitive content, makes ordering
and failure behavior host-global, and cannot preserve terminating observations
reliably. It conflicts with the SDD and is rejected.

### E. Ignore non-terminating messages and implement only termination

This could increase corpus execution quickly but would silently discard a
standard observable and create false host parity. It is rejected.

## Findings and uncertainties

The engine already has the right broad ownership boundary: messages belong to
an invocation and the host decides their operational use. The corpus justifies
a private semantic implementation and static-validation tranche, but not a
public callback or async API.

The first implementation should use a complete, bounded retained collection as
the reference path. It must preserve messages on a terminating outcome, charge
construction and retention independently, and keep message content out of the
principal result tree. A later incremental experiment may be valuable for
large or frequent messages, especially through isolated workers, but has not
earned admission.

Open questions include:

- whether the retained semantic form should be a node sequence, a temporary
  tree, a serialized string, or a deliberately smaller observation type;
- which message limits belong in the private execution policy and how hosts
  eventually supply them;
- when an observation is considered delivered across native and isolated
  boundaries;
- whether a slow or abandoned message consumer retires an isolated worker;
- how XSLT 3.0 `select`, `error-code`, and dynamic termination compose with the
  first XSLT 1.0 content-constructor slice; and
- what bounded metadata may accompany content without leaking internal plans or
  sensitive source values.

## Disposition

**Incubating.** Admit a private bounded retained-message reference experiment
and standards-backed static validation. Do not stabilize a public Rust/.NET
message type, callback, async stream, wire representation, default limit, or
logging policy. Ambient logging and silent discard are rejected.

## Required follow-up

- [x] Classify the 27 OASIS instruction cases by termination, content shape,
  validity, expected operation, and later blocker.
- [x] Validate permitted attributes and XSLT 1.0 `terminate="yes|no"` lexical
  values before reporting unsupported message content.
- [ ] Define a private invocation outcome that can retain bounded ordered
  messages beside success or failure without exposing a public API.
- [ ] Add independent message-count and retained-byte limits, with admission
  before retention and focused exhaustion/cancellation tests.
- [ ] Execute literal text and the already admitted bounded sequence
  constructors through a message-owned semantic sink.
- [ ] Prove `terminate="yes"` preserves the message observation and returns a
  structured standard operation failure without a partial principal result.
- [ ] Compare direct retained-message semantics across native and isolated
  adapters, including transport loss, cancellation, abandonment, and worker
  reuse/retirement.
- [ ] Update AR-0010's message-accounting item when all implemented message
  construction and retention paths are charged.
- [ ] Measure retained and transported message pressure before proposing a
  public delivery surface or example limits.

## Reopening triggers

Move this review toward an ADR when direct semantics, bounded accounting, and
native/isolated parity are executable, or when a real consumer supplies a
message-delivery requirement. Reconsider the retained reference if evidence
shows it cannot bound memory or preserve required observation latency.

## Review history

- 2026-09-23 -- Opened as Incubating from the 27-case OASIS XSLT 1.0 frontier,
  the SDD's explicit message ownership, and AR-0010's unfinished message
  accounting. Selected a private retained collection as the reference
  experiment while rejecting ambient logging and silent discard.
- 2026-09-23 -- Inventoried 23 valid ordinary messages, two invalid
  `terminate` lexicals, and two unknown-attribute cases. Added XSLT 1.0-only
  static validation: the valid runtime frontier remains explicit while four
  malformed cases now receive their earlier classifications.
