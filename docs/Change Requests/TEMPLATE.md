# CR-NNNN: Consumer and Requested Boundary

| Field | Value |
| --- | --- |
| Status | Proposed |
| Requested by | Application, adapter, or sibling project |
| Opened | YYYY-MM-DD |
| Last reviewed | YYYY-MM-DD |
| Target | Milestone or FastXSLT boundary |
| Consumer owner | Person or project responsible for consumer semantics |
| Related reviews | None |
| Related ADRs | None |
| Related plans | None |

## Consumer problem

Describe the workload or integration pressure in consumer language. State why
the consumer cannot safely or efficiently use the present boundary. Do not
prescribe FastXSLT internals unless the observed constraint genuinely requires
it.

## Consumer pipeline

```text
Consumer-owned input and policy
    -> consumer adapter
    -> FastXSLT public contract
    -> compile and transform semantics
    -> structured result and diagnostics
    -> consumer-owned publication
```

Adjust the flow and identify where ownership changes.

## Ownership boundary

### Consumer owns

- Host lifecycle, deployment, and domain meaning.
- Resource acquisition and authority granted to the engine.
- Presentation, logging, and output publication policy.

### FastXSLT owns

- The admitted XSLT, XPath, and XDM semantics.
- Resource identity and lookup inside the supplied engine boundary.
- Structured engine diagnostics and documented limit behavior.

### FastXSLT must not need to understand

- Consumer-only framework, request, database, UI, or deployment types.
- Domain meaning that is not part of the selected standards profile.

## Requested contract

Describe the smallest public behavior the consumer needs. Cover lifecycle,
ownership, cancellation, concurrency, limits, diagnostics, and result transfer
where applicable. Separate required behavior from a preferred binding shape.

## Existing evidence

List a reproducible consumer fixture, current workaround, profile, failure,
latency measurement, allocation result, file-lock observation, or other concrete
pressure. Record missing evidence explicitly.

## Acceptance evidence

| Case | Pressure | Expected result | Evidence |
| --- | --- | --- | --- |
| Representative success | Consumer workload | Structured successful result | Pending |
| Invalid input | Boundary validation | Typed failure or diagnostic | Pending |
| Unsupported behavior | Standards boundary | Distinct unsupported result | Pending |
| Denied resource | Host authority | No ambient fallback | Pending |
| Limit or cancellation | Resource policy | Bounded classified termination | Pending |

Add interop, concurrency, batch, handle-release, compatibility, and performance
rows when the request depends on them. Remove genuinely irrelevant rows with a
short explanation.

## Compatibility and migration

State effects on Rust APIs, host bindings, diagnostic identifiers, serialized
data, compiled artifacts, fixtures, and existing consumers. Write `No impact`
only after checking every applicable boundary.

## Security and resource limits

Identify untrusted inputs, granted capabilities, denied ambient access,
resource budgets, sensitive diagnostic fields, and host callbacks. Keep claims
within the current security policy and safety/limits page.

## Explicit non-goals

- Consumer-specific behavior FastXSLT must not absorb.
- Attractive adjacent work excluded from this request.
- Guarantees not supported by current evidence.

## Proposed disposition

Choose accept, investigate through a named AR, defer with a reopening trigger,
reject with rationale, or supersede with a link. Acceptance should name the plan
that will turn the request into compiling vertical slices.

## Completion condition

State the repeatable consumer-visible evidence required to mark this request
Implemented. Completion of a CR does not create a broader conformance,
performance, or compatibility claim unless an owning specification says so.

## History

- YYYY-MM-DD -- Opened as Proposed.
