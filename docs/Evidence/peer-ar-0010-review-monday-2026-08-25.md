# Peer AR-0010 Review: Monday

| Field             | Value                                                  |
| ----------------- | ------------------------------------------------------ |
| Date              | 2026-08-25                                             |
| Reviewer          | Monday                                                 |
| Reviewed revision | `2ba41c8`                                              |
| Subject           | AR-0010 execution supervision and isolation boundary   |
| Outcome           | Retain Incubating disposition without revision         |
| Informs           | AR-0010 and its planned invocation-control experiments |

## Review conclusion

The review found AR-0010 appropriately scoped and recommended leaving it
Incubating. In particular, the record replaces an unsafe “watchdog kills rogue
threads” intuition with explicit supervision and guarantee classes:

```text
structural/work budget -> engine protection
cancellation           -> cooperative control
deadline               -> operational safeguard
panic containment      -> conditional recovery
process termination    -> hard containment
```

This separation is important for future ASP.NET adapters. A host timeout value
must not imply that native in-process work is guaranteed to have terminated or
that its resources were reclaimed at an exact deadline.

## Strengths confirmed

- Direct execution remains the semantic reference for future dispatched and
  isolated modes. Operational failures specific to transport, worker crash, or
  hard timeout may differ, but transformation semantics and shared diagnostic
  meaning require parity evidence.
- Work accounting belongs in the layer performing the work. XML, XDM, XPath,
  template execution, and serialization charge their own meaningful units; the
  supervisor supplies and observes invocation control without estimating
  semantics from elapsed time.
- Catching an unwind is not worker resurrection. Post-panic disposition may
  require discarding invocation state, retiring a worker, invalidating shared
  state, or replacing an isolated process depending on evidence.
- The AR preserves a future hard-isolation seam without adding speculative
  process machinery to the initial hot path.

## Experimental pressure

The reviewer identified budget composition as the likely next design pressure.
Some limits are naturally structural, including input bytes, XML depth, node
count, recursion, and output bytes. Abstract evaluation work needs deliberate
weighting: charging every XPath operation equally would undercount scans and
other size-dependent work, while excessively precise metering could impose an
unacceptable hot-path cost.

The review therefore supports AR-0010's current refusal to define work units or
accounting granularity before measurement. The next useful evidence is the
charge-point inventory plus adversarial and fault-injection experiments already
listed in the AR, not additional prose or an accepted dispatcher design.

## Limitations

This is a design review, not implementation, security-audit, cancellation
latency, panic-recovery, process-isolation, or performance evidence. It creates
no public timeout, worker-reuse, hard-termination, or hardened-mode guarantee.
