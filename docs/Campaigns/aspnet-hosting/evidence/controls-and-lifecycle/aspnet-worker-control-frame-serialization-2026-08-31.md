# ASP.NET Worker Control-Frame Serialization

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Scope | Private isolated-worker ASP.NET workbench transport |
| Review pressure | Adversarial review Finding 10 |
| Claim | Concurrent outbound cancellation commands retain complete frame boundaries |

## Finding

The controlled-transform gate remains held until the invocation completes, so
active cancellation must bypass that gate. The client previously wrote a cancel
command as separate opcode, length, identity, and flush operations. Its ordinary
handle makes matching cancellation one-shot, but the experimental unrelated
signal can overlap it. `Stream` does not make that multi-write sequence atomic.

The client now delegates cancellation commands to a dedicated outbound control
writer. It UTF-8-encodes and bounds the identity, assembles one complete command
frame, then holds an async serializer across the frame write and flush. This
does not change completion-wins behavior, request correlation, or cancellation
ordering semantics.

## Stress evidence

The workbench operational gate submits 10,000 pairs (20,000 total commands) to
one serializer concurrently. The capture stream yields after accepting every
byte. The verifier parses the resulting bytes as worker protocol commands and
requires exactly one intact frame for every unique expected request identity.

| Observation | Result |
| --- | ---: |
| Concurrent pairs | 10,000 |
| Expected frames | 20,000 |
| Parsed frames | 20,000 |
| Captured bytes | 380,000 |
| Missing, duplicate, interleaved, or malformed frames | 0 |

The complete ASP.NET operational suite also passed. Its existing live-worker
active-cancellation experiment sends an unrelated signal, confirms it does not
complete the target invocation, sends the correlated signal, receives exact
`FXCT0001 / cancelled`, and executes a recovery transform in the same process.
The synthetic stress establishes write serialization; the live experiment
establishes worker protocol interpretation and reuse. Neither is a public
concurrency or cancellation-latency guarantee.

## Validation

- Release ASP.NET workbench build: passed with zero warnings.
- `scripts/verify-aspnet-workbench.ps1 -OperationalExperiments
  -MeasurementRequests 10 -MeasurementRuns 1`: passed.
- The stress probe is part of `-OperationalExperiments` and fails unless all
  20,000 expected identities are recovered exactly once.

## Disposition

Adversarial review Finding 10 is complete for the private workbench. Any future
protocol command that must bypass the ordinary request gate must use the same
frame-level serializer or provide equivalent atomicity evidence.
