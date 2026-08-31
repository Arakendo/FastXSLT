# Native Registry Abandonment Measurement

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Scope | Test-only AR-0017 native registry accounting |
| Workload | 100,000 controls, then 100,000 bounded failure outcomes |
| Mode | Optimized sacrificial Rust test process on Windows |
| Claim | Abandonment and release-retention evidence; no quota or allocator-exact accounting selected |

## Method

Test-only observation reports registry cardinality, `HashMap::capacity`, and
owned outcome payload bytes. A release-mode ignored test records process working
set through a separate PowerShell query, abandons 100,000 controls, releases all
of them, then repeats the process with 100,000 identical bounded 80-byte failure
outcomes. Both empty registry owners remain alive through the final sample so
retained capacity remains observable.

## Observations

| Checkpoint | Logical entries | Map capacity | Outcome payload | Working set |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 0 | 0 | 0 | 4,997,120 B |
| Controls retained | 100,000 | 114,688 | 0 | 14,524,416 B |
| Controls released | 0 | 69,777 | 0 | 10,575,872 B |
| Outcomes retained | 100,000 | 114,688 | 8,000,000 B | 26,599,424 B |
| Outcomes released | 0 | 70,678 | 0 | 15,941,632 B |

On this one run, abandoned controls added 9,527,296 bytes of process working
set, approximately 95.3 bytes per retained control including map/allocation
effects. Outcomes added 16,023,552 bytes above the post-control-release point,
approximately 160.2 bytes per outcome, of which exactly 80 bytes per outcome
were owned encoded payload.

Every release succeeded and logical cardinality plus payload bytes returned to
zero. Working set did not return to baseline: the standard maps retained
substantial effective capacity after removals and the process allocator did not
return all freed storage to the operating system. `HashMap::capacity` may fall
as removals create tombstones, so it is an effective insertion capacity rather
than allocated-byte accounting.

## Interpretation limits

- Working set is process-wide and includes test/runtime/PowerShell-query
  effects; deltas are observations, not allocator-exact attribution.
- The control and outcome phases share one process, and the emptied control map
  remains alive during the outcome phase.
- Outcomes use one small bounded diagnostic shape. Results near the 1 MiB
  per-object maximum would be dominated by payload bytes.
- No prepared engines were abandoned in this probe.
- This is abuse pressure, not Monday's required legitimate live-use high-water
  measurement. It cannot calibrate a production threshold by itself.

The result confirms that explicit release preserves handle semantics but does
not guarantee immediate process-memory reclamation. Future policy comparison
must therefore consider retained empty-map capacity, optional explicit shrink
or bulk domain retirement, and process recycling independently of live-entry
admission. None is selected here.

## Reproduction

```text
cargo test --release -p fastxslt-dotnet-workbench measure_control_and_outcome_registry_abandonment --all-features -- --ignored --nocapture
```
