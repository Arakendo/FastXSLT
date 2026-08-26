# Private Ledger Conservation Experiment

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Implementation | `crates/fastxslt/src/verification_ledger_conservation_experiment.rs` |
| Decision pressure | AR-0011 denominator conservation and reproducible merging |
| Claim | Private accounting evidence; no report schema or conformance claim |

## Method

The experiment uses a nine-case synthetic inventory covering every current
selection bucket. Five cases are selected for execution; four remain visible as
profile-excluded, engine-unsupported, harness-unsupported, and metadata-failed.
The selected cases are divided across two shard reports and cover pass,
semantic mismatch, engine failure, harness failure, and interrupted execution.

The merger derives totals from the complete inventory rather than from emitted
test results. A selected case without a final observation is therefore
explicitly incomplete instead of disappearing. A retry carries a higher
attempt ordinal and supersedes the interrupted observation. Equal attempt
ordinals with contradictory outcomes fail the merge. Execution observations
for unselected or unknown cases also fail rather than changing a denominator.

Each shard carries one run identity and shard topology. Mixed identities,
mixed shard counts, invalid coordinates, and duplicate inventory identities are
rejected. The experiment merges the original shards and retry in forward and
reverse completion order and requires structurally identical ledgers.

## Results

The interrupted ledger conserves:

```text
9 discovered = 5 selected + 1 profile-excluded + 1 engine-unsupported
             + 1 harness-unsupported + 1 metadata failure

5 selected = 1 passed + 1 semantic mismatch + 1 engine failure
           + 1 harness failure + 1 incomplete
```

After retry, the same selection denominator remains and execution conserves:

```text
5 selected = 2 passed + 1 semantic mismatch + 1 engine failure
           + 1 harness failure + 0 incomplete
```

Forward and reverse merge orders produce the same final case outcomes and
totals. The focused tests pass, bringing the workspace to 46 tests: 45 passing
and one ignored manual accounting-cost probe.

## Limitations

- The inventory is deliberately synthetic and small; it does not load all
  46,421 admitted upstream cases.
- Attempt ordinals and latest-attempt precedence are experimental private
  mechanics, not an accepted persistent schema or retry policy.
- The experiment does not yet define immutable report identity fields beyond a
  minimal run identity and shard topology.
- It does not measure memory, runtime, serialization, signing, or storage cost.
- AR-0001 remains open, so the selection counts do not represent an accepted
  standards profile.
