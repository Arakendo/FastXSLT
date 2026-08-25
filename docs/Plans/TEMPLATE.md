# Plan Title

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | YYYY-MM-DD |
| Last updated | YYYY-MM-DD |
| Owner | Maintainer or working group |
| Target | Milestone, layer, adapter, or consumer |
| Related ADRs | None |
| Related reviews | None |
| Related change requests | None |
| Depends on | Existing capability, evidence, or external work |

## Purpose

State the concrete outcome and why FastXSLT needs it now. Name the standards,
consumer, correctness, security, compatibility, performance, or maintenance
pressure rather than only naming a feature.

## Trigger and evidence

Record failing or missing behavior, corpus cases, differential results,
profiles, diagnostics, consumer requests, or repeated implementation friction.
Separate observed behavior from an accepted guarantee and name missing evidence.

## Current state

Describe what exists today, including public and private boundaries, tests,
diagnostics, limits, known unsupported behavior, and relevant specifications.

## Goals

- Goal one.
- Goal two.

## Non-goals

- Adjacent work this plan must not absorb.
- Guarantee the current evidence cannot support.

## Ownership and dependency boundary

Name the layers that own semantics, resource authority, host policy, and
presentation. State permitted dependency direction and forbidden upward,
cyclic, parser-owned-semantic, or consumer-specific dependencies.

### This work owns

- Semantics and decisions introduced by this plan.

### This work must not own

- Higher-level consumer domain meaning.
- Unrelated standards or host policy.
- A public abstraction with no exercised caller.

## Public contract impact

Identify proposed public types, methods, diagnostic identities, host-binding
shapes, serialized data, and compatibility behavior. Mark provisional contracts.
If the plan would decide an unresolved architectural question, stop and open or
update an AR and ADR before implementation treats the choice as accepted.

## Implementation slices

Each slice must compile, preserve accepted behavior, and leave the repository
coherent. Prefer the smallest end-to-end semantic result over horizontal
placeholder layers.

### Slice 0: Baseline and boundary confirmation

**Objective:** Reproduce the pressure and confirm its accepted owner.

#### Deliverables

- [ ] Read relevant specifications, ADRs, ARs, CRs, and corpus policy.
- [ ] Retain a focused fixture, diagnostic, profile, or consumer measurement.
- [ ] Identify standards, public API, authority, concurrency, and compatibility
      impact.

#### Acceptance criteria

- [ ] Evidence is reproducible or its limitations are explicit.
- [ ] Observation and guarantee are distinguished.
- [ ] No implementation silently resolves an open architecture question.

#### Validation

```text
Commands and retained artifacts.
```

#### Exit state

Describe the stable state that permits implementation to begin.

### Slice 1: Smallest useful vertical behavior

**Objective:** Prove one narrow contract from input admission through result or
structured failure.

#### Deliverables

- [ ] Add the smallest compiling implementation.
- [ ] Add focused unit, golden, and boundary tests that genuinely apply.
- [ ] Preserve source provenance and structured diagnostics.
- [ ] Exercise the behavior through one real facade or consumer-shaped caller.

#### Acceptance criteria

- [ ] Supported behavior succeeds deterministically for the admitted case.
- [ ] Invalid, unsupported, denied, exhausted, and internal states used by the
      slice are not collapsed into display text.
- [ ] Engine-owned execution performs no ungranted I/O.
- [ ] A semantic result is distinguishable from serialization behavior.

#### Validation

```text
cargo test --workspace --all-features
```

#### Exit state

Name the useful capability and intentionally unsupported behavior.

### Slice 2: Hardening and independent pressure

**Objective:** Test the boundary against another caller, processor, malformed
case, limit, or representative workload.

#### Deliverables

- [ ] Add adversarial, differential, limit, cancellation, or concurrency cases
      relevant to the work.
- [ ] Exercise another independent caller or explain the evidence substitution.
- [ ] Measure consumer-visible costs when performance motivates the work.
- [ ] Update owning contracts and public summaries.

#### Acceptance criteria

- [ ] Failures remain structured, bounded where promised, and inspectable.
- [ ] Optimization preserves reference semantics.
- [ ] Compatibility and migration behavior are explicit.
- [ ] Private AST, XDM storage, IR, cache, and adapter details do not leak into
      public contracts accidentally.

#### Validation

```text
./scripts/verify.ps1
```

#### Exit state

State whether the work is complete, incubating, parked, or has exposed a new
architectural question.

## Validation matrix

| Concern | Evidence | Command or artifact | Required result |
| --- | --- | --- | --- |
| Unit semantics | Focused tests | Name | Pass |
| End-to-end behavior | Golden or corpus case | Name | Exact classified result |
| Invalid and unsupported input | Negative cases | Name | Distinct diagnostics |
| Resource authority | Denied/missing resource case | Name | No ambient fallback |
| Limits and cancellation | Boundary cases | Name | Documented result |
| Differential behavior | Named processor and version | Report | Explained match/divergence |
| Consumer integration | Rust or host workbench | Report | Contract exercised |
| Performance | Correctness-gated benchmark | Report | Baseline retained |
| Documentation | Local gate | `./scripts/verify.ps1` | Pass |

Remove rows that do not apply and explain why. Add handle-release, batch,
threading, FFI, Miri, sanitizer, fuzz, or memory rows when the plan affects them.

## Failure and diagnostic semantics

List expected semantic findings and operation failures, their owning layers,
stable identities if implemented, structured details, and presentation
boundaries. No expected state should rely only on a log line, panic, or silent
fallback.

## Compatibility and migration

State effects on the standards profile, Rust API, host ABI, diagnostic codes,
serialized results, compiled artifacts, fixtures, and consumers. FastXSLT has no
stable compiled-artifact format today; introducing one requires deliberate
versioning and migration decisions.

## Security and resource bounds

State changes to untrusted-input parsing, resolvers and host capabilities,
external entities, extensions, source disclosure, cancellation, and resource
limits. Identify whether each promised limit is deterministic, best-effort, or
host-enforced.

## Performance and memory

Record preload, parse, compile, warm execution, serialization/result transfer,
interop, retained memory, and peak memory where relevant. Performance evidence
is not a guarantee unless an accepted contract makes it one.

## Risks and mitigations

| Risk | Impact | Mitigation or evidence |
| --- | --- | --- |
| Example | Consequence | Test, bound, diagnostic, or design response |

## Completion, parking, and reopening

The plan is complete only when in-scope acceptance criteria pass, validation is
repeatable, public behavior and unsupported cases are documented, and remaining
work has an owner and destination. If parked, name why useful work should stop
and the observable evidence that would reopen it.

## Progress log

### YYYY-MM-DD

- Work completed:
- Validation:
- Findings:
- Plan changes:
- Next slice:

Append entries. Do not rewrite history to make implementation look linear.

## References

- Owning specification, ADRs, ARs, CRs, tests, fixtures, and evidence.
