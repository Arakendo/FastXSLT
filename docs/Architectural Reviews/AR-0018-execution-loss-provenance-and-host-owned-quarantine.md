# AR-0018: Execution-Loss Provenance and Host-Owned Quarantine

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-09-03 |
| Last reviewed | 2026-09-03 |
| Scope | Attempt identity, execution-loss observations, durable host correlation, retry, and quarantine |
| Trigger | A field host must prevent one repeatedly catastrophic stylesheet/input combination from cycling through replacement workers indefinitely |
| Related ADRs | ADR-0002, ADR-0005, ADR-0016 |
| Related reviews | AR-0002, AR-0004, AR-0005, AR-0010 |
| Related evidence | `docs/Evidence/aspnet-worker-recovery-and-generation-replacement-2026-08-26.md`; `docs/Evidence/aspnet-host-mode-guarantee-cost-matrix-2026-08-26.md`; `docs/Evidence/peer-execution-loss-quarantine-review-monday-2026-09-03.md`; future attempt-journal and fault-injection evidence |

## Architectural question

What bounded, host-neutral execution observations should FastXSLT expose when
an invocation loses its worker or containment boundary, so a host can correlate,
persist, retry, quarantine, alert, or escalate the affected work without turning
FastXSLT into a durable scheduler or falsely classifying source data as bad?

## Trigger and evidence

The isolated ASP.NET workbench already demonstrates one important conservative
behavior. When a non-cooperating invocation is acknowledged and its worker is
forcibly terminated, the host reports `FXWB2001 / worker-terminated`, does not
retry the ambiguous invocation, replaces only the affected slot, and proves a
sibling plus later work can complete. This is fault-containment evidence, not a
production retry or quarantine contract.

A field queue commonly retries work after process loss. Without stable request,
attempt, stylesheet-generation, snapshot, policy, and worker-generation
correlation, the same work can repeatedly kill replacement workers while the
queue treats every death as an unrelated transient failure. Conversely, one
worker death does not prove that the XML or stylesheet caused it. The process
could have been killed by the host, exhausted by aggregate load, affected by an
engine defect, or lost during deployment.

ADR-0005 already assigns retry, rollback, publication, transactions, and
workflow stages to the host. ADR-0016 establishes the broader host-owned policy
principle for operational values. AR-0010 distinguishes structured engine
failure, cooperative cancellation, deterministic limits, best-effort deadlines,
panic, transport loss, and hard process termination. What remains unknown is
the smallest observation envelope and lifecycle protocol a real host needs to
apply those accepted ownership boundaries safely.

No current evidence establishes a durable event schema, exactly-once delivery,
crash-safe host journal, retry threshold, quarantine duration, escalation key,
privacy profile, or public Rust/.NET API.

## Ownership and constraints

FastXSLT and its host adapter may own:

- bounded machine-readable observations about an invocation it accepted;
- stable correlation of an observation with host-supplied logical request and
  attempt identities;
- distinctions among semantic completion, structured engine failure,
  cooperative cancellation, deterministic limit exhaustion, deadline signal,
  worker crash, hard termination, and transport loss;
- explicit execution-mode and worker-generation correlation where the adapter
  owns those concepts;
- a small, versioned last-known phase vocabulary whose values are semantic and
  do not expose private AST, arena, cache, or optimizer structure; and
- conservative `unknown` or `ambiguous` states when a process boundary cannot
  prove whether execution or result transfer completed.

The host owns:

- durable storage, transaction boundaries, retention, encryption, and access
  control for operational records;
- request, attempt, publication, tenant, stylesheet-generation, snapshot, and
  policy identities supplied to the adapter;
- automatic-retry eligibility and limits, backoff, quarantine, alerting,
  operator workflow, and escalation scope;
- deciding whether repeated loss implicates one request, one stylesheet
  generation, a snapshot, a tenant, a worker image, or the service itself;
- idempotent result publication and reconciliation after ambiguous completion;
  and
- choosing native or isolated execution and any external process/container
  memory or time ceiling.

The following constraints apply:

- Engine execution performs no hidden durable I/O. An observation sink or
  journal is supplied and operated by the host.
- A URL, filename, process ID, content fingerprint, or destination is not
  resource authority or durable request identity.
- Content fingerprints are host-safe correlation hints only. They do not prove
  XDM identity, semantic equivalence, or authority and must not become hidden
  cross-generation cache keys.
- Raw XML, stylesheet bytes, parameter values, source snippets, host paths,
  secret-bearing URIs, or unbounded diagnostic text must not be required in a
  trace envelope.
- Every field crossing a boundary is bounded. A trace must not become a new
  denial-of-service or information-disclosure channel.
- Result production does not imply result publication. A worker can disappear
  after computing a result but before the host durably commits it.
- FastXSLT cannot guarantee a final worker-emitted event after process death.
  The supervisor may synthesize an execution-loss observation from its last
  durable or in-memory knowledge.
- No automatic retry is hidden inside FastXSLT. A retry is a new attempt with a
  distinct attempt identity, even when it retains the same logical request.
- Quarantine means that host policy requires different handling. It is not a
  semantic finding that the customer's input is invalid or malicious.
- Host-selected retry counts, time windows, and escalation thresholds follow
  ADR-0016. FastXSLT must not invent universal defaults.

## Candidate observation model

The first experiment should test a bounded versioned envelope rather than
stabilize one. Candidate fields are:

- schema/version identity;
- host-supplied logical request and unique attempt identities;
- host-supplied stylesheet generation, resource snapshot generation, and
  operational-policy identity;
- execution mode plus host/adapter-assigned worker slot and worker generation;
- lifecycle transition and a monotonic per-attempt observation sequence;
- last-known coarse phase, such as admitted, initializing, compiling,
  preparing, executing, serializing, transferring result, or unknown;
- disposition and termination origin;
- bounded structured diagnostic identity when one exists;
- host-observed start/end timestamps and monotonic duration; and
- optional host-generated opaque fingerprints that reveal no source content.

An operating-system process identifier may be useful transient debugging data,
but it is recyclable and must not be the durable worker-generation identity.
Likewise, last-known phase is evidence about the latest observation, not proof
that the worker died inside that phase. It may be stale or unknown.

The minimum lifecycle must distinguish request identity from attempt identity:

```text
logical request R
    |
    +-- attempt A1 -> worker lost -> ambiguous -> quarantined by host
    |
    +-- attempt A2 -> only if host policy explicitly retries
```

This permits a host to detect repeated equivalent-condition worker loss without
erasing the history of each containment event.

## Alternatives

### A. Retain only the current per-call result or transport exception

This keeps the boundary small but leaves durable queues unable to distinguish a
new attempt from repeated catastrophic work after a host or worker restart. It
also encourages display-string parsing and accidental blanket retries.

### B. Expose bounded attempt observations; host owns persistence and policy

FastXSLT and the adapter emit or return structured facts. The host brackets
dispatch with its own durable attempt record, consumes observations
idempotently, reconciles orphaned running attempts after restart, and selects
retry or quarantine. This is the leading experiment because it extends existing
identity and diagnostic boundaries without moving workflow authority.

It cannot promise exactly-once observation delivery. A host that needs crash
recovery must durably record intent before dispatch and treat an unmatched
running attempt as ambiguous until its own reconciliation policy resolves it.

### C. FastXSLT owns a durable journal or dead-letter queue

An engine-owned database/spool could centralize recovery, but would introduce
filesystem or network authority, persistence schemas, transactions, retention,
credentials, encryption, deployment, and workflow policy. It conflicts with
ADR-0002's memory-resident default and ADR-0005's host-owned workflow boundary.

### D. FastXSLT automatically retries worker loss

Transparent retry can hide transient failures, but repeats potentially harmful
work, loses attempt provenance, complicates publication exactly-once claims, and
silently chooses host availability policy. It is not compatible with the
current conservative workbench behavior.

### E. Quarantine every input after one worker loss

This stops a crash loop quickly but assigns causation without evidence. It can
misclassify valid work after deployment faults, aggregate OOM, host termination,
or engine defects. A host may deliberately choose this policy for a trust
profile, but FastXSLT should not encode it as semantics.

## Findings and uncertainties

- The current request identity is necessary but insufficient. Retries require
  a unique attempt identity and an explicit parent logical request.
- A terminal structured XSLT failure is not the same as execution loss. The
  former can be reported by the engine; the latter may have only supervisor
  evidence and an ambiguous completion state.
- The worker cannot reliably deliver its own death record. A useful design must
  allow the supervisor/host to close or reconcile an attempt from last-known
  facts.
- Durable recording must precede dispatch if a host wants to discover orphaned
  attempts after its own restart. FastXSLT cannot supply that durability from
  inside the worker.
- Exactly-once execution is not established and may be impossible to observe
  across process loss. Idempotent host publication and attempt-aware
  reconciliation are more defensible targets.
- A coarse phase may improve forensics, but updating it too frequently can add
  hot-path coordination and leak implementation detail. No phase vocabulary or
  observation frequency is selected.
- The correlation key for repeated loss is policy-sensitive. Same request,
  stylesheet generation, snapshot, policy, engine version, and worker image may
  all matter; FastXSLT should provide facts without declaring one universal
  poison key.
- Per-request quarantine is the narrowest likely host action. Escalation to a
  stylesheet generation, publication, tenant, or deployment needs repeated or
  independently corroborating evidence.
- Diagnostics and phase metadata may contain sensitive information even when
  source bytes are absent. A redaction and maximum-size profile needs testing.
- The public Rust facade, managed adapter, event delivery shape, and durable
  schema remain unresolved by AR-0002, AR-0004, AR-0005, and AR-0012.

## Disposition

**Incubating.** Preserve a host-owned attempt/journal/quarantine model and test a
bounded execution-loss envelope through the isolated ASP.NET workbench. Do not
select a durable schema, retry threshold, quarantine policy, callback API, or
public guarantee until a representative host demonstrates its transaction and
recovery requirements.

FastXSLT should continue to return ordinary structured semantic outcomes where
possible. Worker loss after admission is conservatively ambiguous unless the
boundary can prove a narrower state. The host decides whether that attempt is
retried, quarantined, escalated, or abandoned.

## Required follow-up

- [ ] Inventory every current point where the ASP.NET supervisor knows a request
  was queued, assigned, acknowledged, executing, completed, transferred, or
  lost, and identify which observations survive worker death.
- [ ] Add host-supplied attempt identity alongside logical request identity in a
  private workbench experiment without changing XSLT semantics.
- [ ] Define a bounded candidate envelope and prove it excludes raw source,
  stylesheet, parameter, path, URI-secret, and unbounded diagnostic content.
- [ ] Fault-inject loss before admission, after assignment, after worker
  acknowledgement, during execution, during result transfer, and after result
  receipt but before simulated publication.
- [ ] Prove the host records each retry as a new attempt and never performs an
  implicit FastXSLT retry.
- [ ] Run one repeatably catastrophic work item across replacement workers and
  prove host policy can quarantine it while unrelated siblings continue.
- [ ] Inject a worker/deployment-wide failure to prove quarantine does not
  automatically convict an individual XML document.
- [ ] Restart the host with an attempt left in `running` and demonstrate
  deterministic reconciliation to an explicit ambiguous disposition.
- [ ] Exercise duplicate, missing, delayed, and out-of-order observations and
  prove host journal updates are idempotent and monotonic per attempt.
- [ ] Measure observation volume, retained metadata, serialization cost, and
  hot-path overhead under bounded concurrent execution.
- [ ] Compare callback, returned-envelope, pull/status, and supervisor-event
  delivery shapes against the eventual Rust and .NET facade requirements.
- [ ] Feed the resulting failure identities into AR-0004 and the read-only
  inspection requirements into AR-0005 before accepting a public contract.
- [ ] Threat-model cross-tenant identity disclosure, fingerprint linkability,
  diagnostic redaction, journal poisoning, and forged/replayed observations.

## Reopening triggers

Advance this review when a representative host supplies its durable queue,
transaction, publication, retry, and recovery semantics; when a real workload
causes repeated worker loss; when the public host facade needs an observation
contract; or when process/container supervision introduces a new failure state
that cannot be represented by the candidate envelope.

Reject or narrow the candidate if useful correlation requires raw customer
content, if the observation path materially harms transform latency, or if the
host can meet the same need entirely through existing structured outcomes and
its own dispatch journal.

## Review history

- 2026-09-03 -- Opened as Incubating from field-operation pressure to prevent
  repeatedly catastrophic work from cycling through replacement workers while
  retaining host ownership of durability, retry, and quarantine policy.
- 2026-09-03 -- Peer review confirmed the request/attempt split, ambiguous
  completion model, bounded privacy posture, host-owned quarantine boundary,
  and fault-injection plan. Retained Incubating and explicitly kept the first
  phase vocabulary coarse pending measured operational need.
