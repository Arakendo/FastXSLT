# ASP.NET Worker Recovery and Generation Replacement

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Host | ASP.NET Core targeting .NET 8 on Windows |
| Engine path | Persistent isolated `fastxslt-worker` processes |
| Stylesheet | Pinned XSLT30 `for-004` |
| Command | `./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 10 -MeasurementRuns 1` |
| Claim | Private lifecycle and fault-containment evidence; not a production supervisor contract |

## Worker termination and replacement

A bounded two-worker pool submitted one ordinary sibling request and one
workbench-only non-cooperating request. The worker acknowledged the latter's
logical identity and deliberately stopped making progress. The host forcibly
terminated only that process, classified the affected invocation as
`FXWB2001 / worker-terminated`, did not retry the ambiguous invocation, and
initialized a replacement from the same sealed source and stylesheet bytes.

The verifier confirmed that the replacement had a distinct process identity.
The sibling completed with the exact expected serialization, as did a later
request assigned to the replacement. Process identifiers vary by run and are
observations, not resource or generation identities.

This demonstrates that a non-cooperating isolated process can be reclaimed
without poisoning this private supervisor or its sibling worker. It does not
establish a timeout duration, cancellation observation bound, retry policy,
crash-loop policy, sandbox strength, or tenant-security guarantee. The probe is
deliberately separate from cooperative cancellation.

## Explicit generation promotion

The host started `generation-001`, acquired an old-generation lease, fully
initialized `generation-002`, and then atomically promoted it. A new request was
correlated with `generation-002`; the acquired old request still executed on
`generation-001`. The retired generation remained alive until its lease was
released and was then disposed.

Generation identities were supplied explicitly by the host. They were not
derived from filenames, paths, process identifiers, or content hashes. Resource
bytes crossed the boundary once per worker generation rather than once per
transform. No result was implicitly admitted as a later resource.

The experiment used identical semantic bytes in the two generations so both
results could be compared exactly. It therefore proves promotion and draining
mechanics, not changed-resource semantics or original-file replacement while
requests are active.

## Disposition

The evidence supports continuing the isolated ASP.NET experiment with distinct
operational failure categories and host-owned generations. AR-0002 remains
Proposed and AR-0010 remains Incubating. Cooperative cancellation, deadlines,
diagnostic parity, panic disposition, restart backoff, and the in-process
FastXSLT comparison remain open.
