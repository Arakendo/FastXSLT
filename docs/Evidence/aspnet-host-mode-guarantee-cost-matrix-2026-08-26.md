# ASP.NET Host-Mode Guarantee and Cost Matrix

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Inputs | Executable ASP.NET evidence through ADR-0008, ADR-0009, and ADR-0010 |
| Compared modes | In-process native library and persistent isolated worker pool |
| Semantic workload | Shared Rust `ExperimentalEngine`; pinned XSLT30 `for-004` plus representative failures |
| Claim | Decision synthesis from private workbench evidence; not a supported-mode decision |

## Shared candidate lifecycle

Both mechanisms now execute the same host-neutral lifecycle:

1. the host imports bounded resource bytes and releases external handles;
2. a candidate generation compiles stylesheets and prepares source-derived XDM
   before promotion;
3. the host atomically promotes an explicit generation identity;
4. identified invocations carry invocation-local cancellation and budgets;
5. results or structured failures retain logical request identity;
6. independent engines/workers supply bounded concurrency; and
7. retired generations drain acquired leases before disposal.

This is the strongest current candidate for a shared managed contract. Engine
handles, worker process identifiers, native control handles, protocol frames,
and pool implementations are mode-specific details and should not appear in
that shared semantic lifecycle.

## Executable guarantee matrix

| Concern | Native in-process | Isolated persistent workers | Shared host-facing meaning |
| --- | --- | --- | --- |
| Engine semantics | Same Rust compiled/prepared engine | Same Rust compiled/prepared engine | One semantic implementation |
| Resource admission | Copied bounded buffers | Bounded frames copied once per worker generation | Host supplies owned qualified resources |
| Ambient file/network authority | None in engine path | None in engine path | Resolver authority remains explicit |
| Compile/prepare reuse | Retained per engine handle | Retained per worker process | Compile/prepare once, transform many |
| Bounded concurrency | Pool of independent handles | Pool of independent workers | No same-engine/worker concurrent contract |
| Result correlation | Logical request identity | Logical request identity | Independent of completion position |
| Structured diagnostics | Current invalid/XML/unsupported/cancelled/limit matrix preserved | Same matrix preserved | Host does not parse display strings |
| Pre-dispatch cancellation | Cooperative scalar control | Cooperative protocol control | `FXCT0001`; ordinary state remains reusable |
| Active cancellation | Rust-owned numeric control handle | Correlated worker control command | Cooperative, completion-wins race |
| Instruction budget | Invocation-local scalar | Invocation-local command field | `FXCT0002`; distinct from cancellation |
| Generation promotion | Managed atomic handle-pool promotion and lease draining | Managed atomic worker-pool promotion and lease draining | Host owns publication and retirement |
| Non-cooperating execution | Cannot reclaim work without ending ASP.NET process | Acknowledged worker can be terminated and replaced | Guarantee differs by deployment mode |
| Panic/crash disposition | Entire native lane quarantines; host process recycle may be required | Failed worker can be replaced without killing a sibling | Operational failure is mode-specific |
| Hard termination | No | Demonstrated per-worker termination | Must never be implied by cancellation |
| Unsafe first-party Rust | Two audited buffer-copy blocks | None in worker boundary | Semantic engine remains safe Rust |
| Packaging | Platform-specific native library plus managed adapter | Worker executable plus managed adapter/protocol | Deployment assets differ |

The isolated fault experiment proves process termination and replacement
mechanics, not sandbox strength, tenant isolation, deadline precision, or a
production restart/backoff policy.

## Measured cost matrix

| Cost | Native in-process | Isolated workers | Evidence boundary |
| --- | --- | --- | --- |
| Tiny warm throughput, 5 items sequential | 347,102/s median | 15,911/s median | Three-run same-workload tier comparison |
| Tiny warm p50, 5 items | 2.4 microseconds | 55.1 microseconds | Includes managed lease/boundary path |
| Larger warm throughput, 500 items sequential | 7,703/s median | 5,166/s median | Boundary ratio narrows to 1.49x |
| Larger warm p50, 500 items | 107.5 microseconds | 186.2 microseconds | Semantic work amortizes transport |
| Four-way throughput scaling | 2.99x to 3.83x over native sequential lanes | Earlier isolated tiers showed 3.27x to 4.13x | Independent handles/workers, synthetic workload |
| Approximate managed allocation | About 454-464 B per invocation | About 3.0 KiB per invocation | Excludes Rust allocations in both modes |
| Prepared-state memory | Whole ASP.NET process is not attributable | Four workers measured about 17.3-21.1 MiB aggregate | No total native allocation comparison yet |
| Active cancellation | Two natural samples observed roughly 0.026-0.491 ms | Earlier isolated sample observed roughly 0.095-0.429 ms | Local observations, not deadline bounds |

The native advantage is largest when fixed process transport dominates. At 500
items the paths are much closer. These synthetic tiers cannot choose a default
for an unknown consumer distribution.

## Candidate profile interpretation

Current evidence supports evaluating two explicit profiles over the shared
lifecycle:

- **trusted low-latency profile:** native in-process execution, cooperative
  controls, lane quarantine, and no hard-kill promise;
- **isolated containment profile:** worker execution with transport cost,
  cooperative controls, and host-authorized worker termination/replacement.

This is an inference from evidence, not an accepted product decision. Calling
the worker profile “untrusted” or “sandboxed” would overclaim current process
launch and authority evidence. Calling native cancellation a timeout would
overclaim cooperative checks.

## Remaining decision inputs

AR-0002 still lacks representative consumer transforms, source/result-size
distributions, concurrency, deployment targets, trust model, latency/throughput
budgets, and acceptable process-recycle policy. Dedicated cold deployment,
native allocation/retention, transport-stage attribution, sustained load, and
packaging/version-skew evidence also remain incomplete.

Until those inputs exist, FastXSLT should preserve both workbench candidates,
stabilize neither ABI/protocol, and describe only the shared semantic lifecycle
as the leading contract shape.
