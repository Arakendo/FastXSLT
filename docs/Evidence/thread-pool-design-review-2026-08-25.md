# Thread-Pool and Volume-Work Design Review

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Input | Project-owner design conversation about pooled volume execution |
| Scope | Prepared inputs, transform-set construction, scheduling, and workflow ownership |
| Informs | ADR-0005 and AR-0003 |

## Accepted conclusion

The conversation establishes one decision-shaped boundary: a transform set is
independent unordered work, while the host owns sequencing between dependent
stages and explicit promotion of earlier results into later resource snapshots.
ADR-0005 records that conclusion.

This protects sealed snapshots from mutation during execution and lets result
identity, rather than completion position or a filename, correlate work.

## Useful implementation pressure

The following concepts fit the existing resource and runtime ownership, but are
not yet accepted public types:

```text
bounded resource snapshot
       |
       +--> immutable prepared inputs where measured reuse justifies them
       |
compiled stylesheets
       |
       v
validated independent transform set
       |
bounded executor / in-flight work
       |
identified in-memory results
```

- Prepared source state must be immutable and shareable. Transformation-specific
  parameters, context, variables, messages, cancellation, and temporary values
  cannot enter it.
- Raw bytes, parsed XDM, and derived indexes have different memory and invalidation
  costs and should not become one unbounded cache.
- Logical resource identity remains distinct from a content fingerprint. Equal
  bytes do not automatically mean one XDM document identity or base URI.
- The client describes work declaratively and seals it before execution. Queue
  and worker mechanics remain internal so scheduling can evolve.
- Input capacity, queue depth, worker count, and maximum in-flight transforms are
  separate budgets.
- Results require stable logical identity. A name resembling `abc.html` is not
  engine authority to write a file.

## Items deliberately not accepted

- `InputPool`, `WorkSetBuilder`, `TransformSetBuilder`, and `Executor` are
  illustrative names, not API commitments.
- 5,000 inputs and 10 workers are a representative workload, not defaults.
- No evidence selects eager versus lazy source preparation, an eviction policy,
  `Arc`, a queue implementation, work stealing, async execution, or native
  threads.
- Failure collection versus fail-fast, cancellation granularity, result-retention
  policy, and presentation ordering remain open execution contracts.
- A transformation graph remains deferred until a real consumer demonstrates
  material benefit over host-owned stages.

## Implementation timing

FastXSLT cannot meaningfully implement a production worker pool before one
transformation executes and compiled/source state has measured sharing behavior.
AR-0003 remains Under Review for budgets, reuse, executor mechanics, and public
lifecycle. M3 is the earliest roadmap milestone for a correctness-gated
independent batch experiment.
