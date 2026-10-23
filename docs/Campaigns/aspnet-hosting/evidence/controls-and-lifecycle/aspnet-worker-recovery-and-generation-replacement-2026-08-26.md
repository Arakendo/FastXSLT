# ASP.NET Worker Recovery and Generation Replacement

| Field       | Value                                                                                                      |
| ----------- | ---------------------------------------------------------------------------------------------------------- |
| Date        | 2026-08-26                                                                                                 |
| Host        | ASP.NET Core targeting .NET 8 on Windows                                                                   |
| Engine path | Persistent isolated `fastxslt-worker` processes                                                            |
| Stylesheet  | Pinned XSLT30 `for-004`                                                                                    |
| Command     | `./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 10 -MeasurementRuns 1` |
| Claim       | Private lifecycle and fault-containment evidence; not a production supervisor contract                     |

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

The first generation experiment used identical semantic bytes so both results
could be compared exactly. It proves promotion and draining mechanics
independently of a semantic change.

## Host file replacement with changed bytes

A second generation experiment created source and stylesheet files under the
gitignored `.workbench/` area and imported each through an explicitly scoped
host file stream. After the streams closed, the host started
`file-generation-001` and acquired an old-generation lease. While that
generation remained live, Windows permitted both original files to be renamed
and deleted. The host wrote a changed source and replacement stylesheet at the
same paths, imported and closed them, and promoted `file-generation-002`.

The old leased generation returned `<out>1.00</out>` from the original one-item
source. A new request returned `<out>2.00</out>` from the replacement two-item
source. This proves that the worker retains sealed resource bytes rather than a
host path or file handle, and that path reuse does not mutate old-generation
semantics. Scratch files were removed after both generations drained.

This remains adapter evidence. FastXSLT did not gain filesystem authority, a
path-based resource identity, or an automatic file-watching/reload contract.

## Disposition

The evidence supports continuing the isolated ASP.NET experiment with distinct
operational failure categories and host-owned generations. AR-0002 remains
Proposed and AR-0010 remains Incubating. Cooperative cancellation, deadlines,
diagnostic parity, panic disposition, restart backoff, and the in-process
FastXSLT comparison remain open.
