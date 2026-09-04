# Peer Review: Execution-Loss Provenance and Quarantine

| Field | Value |
| --- | --- |
| Date | 2026-09-03 |
| Reviewer | Monday |
| Scope | Field handling after worker crash, hard termination, containment loss, or disappearance during a transform |
| Result | Open AR-0018; retain host-owned durability and quarantine policy |

## Review

A host handling durable transformation work needs enough request and execution
provenance to recognize that an exact work item was active when containment
fired. Blindly returning that item to an ordinary retry loop can repeatedly
destroy replacement workers.

The trace should remain host-owned. FastXSLT supplies structured facts; the
application decides whether they are persisted and whether the attempt is
retried, quarantined, alerted, checkpointed, or abandoned. Useful correlation
may include logical request identity, stylesheet and resource generation,
host-safe fingerprints, policy identity, worker generation, disposition,
structured diagnostics, termination kind, and a bounded last-known phase.

Quarantine must not itself declare the XML invalid or malicious. A worker may be
lost because of an engine defect, deployment corruption, aggregate memory
pressure, host termination, or another cause unrelated to that input. The
narrow initial granularity is one transform request; the host may escalate to a
stylesheet generation or publication only when repeated evidence supports it.

Raw customer XML or stylesheet content is not required in the operational
record. Identity, fingerprints, generation, diagnostics, and bounded metadata
can be retained under the host's existing data-at-rest controls while the
actual resource remains in its authoritative store.

The resulting boundary is:

> FastXSLT produces execution observations. The host turns observations into
> retry, quarantine, alert, worker replacement, checkpoint, and publication
> policy.

AR-0018 carries the unresolved attempt identity, ambiguity, persistence,
privacy, phase vocabulary, and fault-injection questions without expanding the
engine into a job scheduler or database.
