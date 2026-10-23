# AR-0020 Bounded Staged-Trial Replay

| Field   | Value                                                                                                   |
| ------- | ------------------------------------------------------------------------------------------------------- |
| Date    | 2026-09-05                                                                                              |
| Scope   | Offline comparison of combined-10 against staged-7/3 on identical deterministic workloads               |
| Status  | Exploratory policy replay; not a connected adaptive dispatcher                                          |
| Command | `cargo test --release -p fastxslt measures_bounded_staged_trial_policy_replay -- --ignored --nocapture` |

## Question

Can a bounded trial reject staged execution when its measured gain fails to
justify tail-latency or prepared-memory pressure, and what exposure would a
losing trial impose before rollback?

## Method

The ignored release probe runs the same pinned stylesheet and semantically
checked requests through ten combined prepare-then-execute workers and a static
seven-preparer/three-executor candidate. It takes the median elapsed run from
seven samples for each topology, then applies three deliberately explicit
experimental gates:

- staged throughput must be at least 1.05 times combined throughput;
- the worst common per-size p95 must be no more than 1.25 times combined; and
- staged prepared-capacity high-water must be no more than 1.5 times combined.

These values exercise the policy mechanism. They are not selected product
defaults. A normal unit test independently proves that throughput, tail, and
memory can each veto admission.

The probe reports the complete elapsed staged trial as **rejected-trial
exposure**, not as incremental overhead. A real controller would spend that
time operating under the losing candidate before rollback; the counterfactual
increment cannot be known from two non-simultaneous runs.

## Results

Two consecutive release runs produced:

| Workload          | Run | Throughput ratio, staged/combined | Worst p95 ratio | Prepared high-water ratio | Decision | Rejected exposure |
| ----------------- | --: | --------------------------------: | --------------: | ------------------------: | -------- | ----------------: |
| Uniform 50        |   1 |                             1.032 |           1.074 |                     2.600 | Reject   |         15.731 ms |
| Uniform 50        |   2 |                             1.545 |           0.570 |                     2.400 | Reject   |         15.951 ms |
| Uniform 500       |   1 |                             0.857 |           0.967 |                     1.000 | Reject   |         81.108 ms |
| Uniform 500       |   2 |                             0.824 |           0.877 |                     1.000 | Reject   |         82.632 ms |
| Mixed clustered   |   1 |                             0.854 |           5.585 |                     1.000 | Reject   |         56.527 ms |
| Mixed clustered   |   2 |                             0.842 |           6.332 |                     1.200 | Reject   |         56.549 ms |
| Mixed interleaved |   1 |                             0.865 |           7.137 |                     1.315 | Reject   |         56.540 ms |
| Mixed interleaved |   2 |                             0.816 |          10.689 |                     0.850 | Reject   |         59.312 ms |

All transformed results retained the expected semantic sentinel.

## Interpretation

The bounded policy rejected all candidates, but for different reasons.
Uniform-500 and both mixed orders consistently lost throughput. Mixed workloads
also produced severe worst-size tail regressions. Uniform-50 sometimes gained
substantial throughput, but exceeded the experiment's memory-pressure gate in
both runs.

Uniform-50 also exposes a controller problem: its observed throughput ratio
moved from 1.032 to 1.545 across consecutive invocations even though the staged
throughput was comparatively stable. A short paired canary could therefore
reach different conclusions from environmental variation in the combined
reference. One trial window is not a trustworthy activation oracle.

This evidence strengthens combined/local execution as the baseline and
supports immediate rollback gates, but it does not yet admit adaptive staging.
A real controller would need longer or repeated windows, confidence/variance
treatment, and a strictly bounded fraction of work exposed to trials. That
additional complexity must outperform simply retaining combined workers.

The provisional product disposition is therefore to omit adaptive staging from
the prototype path. Further scheduler work is gated on a representative named
consumer workload demonstrating a repeatable deficiency in combined mode. A
synthetic success is no longer sufficient to justify exposing real work to
trial windows.

The implementation remains an offline replay that creates topology-specific
threads for each sample. It does not prove a fixed role-flexible pool, stable
online reassignment, cancellation during transition, or cross-machine
portability.
