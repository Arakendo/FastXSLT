# Native Registry Exhaustion Delivery Comparison

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review | AR-0017 |
| Scope | Reserved structured sentinel versus out-of-band scalar quota-exhaustion delivery |
| Claim | Delivery-shape comparison and nomination only; no quota, ABI revision, or status encoding accepted |

## Existing boundary pressure

Every version-zero create or transform export currently returns one `u64` that
is either a nonzero outcome handle or zero after an unrecoverable boundary
failure. Ordinary invalid input and engine failures are retained as structured
outcomes. That uniform shape stops working when the outcome registry itself is
at capacity: reporting quota exhaustion through another ordinary outcome would
require the capacity whose absence caused the failure.

The comparison preserves these constraints:

- delivery consumes no ordinary engine, control, or outcome capacity;
- the result is correlated with the synchronous call that encountered
  exhaustion and uses no thread-local or process-global last-error slot;
- existing valid handles are neither evicted nor repurposed;
- callers can distinguish exhaustion without parsing prose;
- no registry lock encloses semantic work or foreign-memory access; and
- a decision must not silently make the unpublished version-zero ABI public.

## Candidate A: reserved structured sentinel outcome

One numeric value, such as a permanently reserved handle, could make the
existing outcome inspection operations return a fixed bounded `FXFFI` quota
diagnostic without storing an `Outcome`.

This preserves the current managed flow—inspect kind, copy bytes, release—but
only by giving the sentinel special behavior in every outcome operation. Kind,
length, copy, take-engine, and release must all recognize it outside the map.
Release must either report success for a value that was never owned, or report
failure from an otherwise ordinary `SafeHandle` cleanup. Any foreign caller can
also fabricate the sentinel and observe the same structured diagnostic even
when no admission was attempted. Request identity and dynamic detail cannot be
included without reintroducing mutable storage.

The sentinel is bounded and requires no new pointer operation, but it makes one
value look like an outcome handle while violating normal handle ownership and
release semantics.

## Candidate B: scalar status plus output pointer

A versioned export could return a scalar admission status and write a successful
outcome handle through `u64 *outcome_handle`. Exhaustion would return a fixed
status without creating an outcome.

This creates a clean status/handle distinction and avoids a sentinel, but adds a
new foreign writable-pointer contract to every producing operation. The native
side must validate and initialize the output slot, define its state on every
failure and panic path, and extend ADR-0008's exact unsafe surface. Returning a
C struct would instead add ABI layout surface expressly excluded by ADR-0008.
Thread-local last-error or a second lookup call is rejected because concurrent
callers can lose correlation.

The pointer formulation is viable, but its extra unsafe and ABI surface is not
necessary for a pair of fixed-width values.

## Candidate C: tagged scalar admission result

A versioned operation can partition its existing `u64` return space:

```text
low nonzero range     -> ordinary owned outcome handle
reserved tagged range -> out-of-band admission status
zero                  -> legacy/fallback invalid result, if still required
```

For example, a high tag bit can distinguish a small fixed status code from a
handle while leaving more than enough nonzero handle values for the process
lifetime. The exact bit, codes, and fallback behavior are intentionally not
selected here.

The managed wrapper checks the tag before constructing a `SafeHandle`. Outcome
inspection and release never recognize tagged statuses as handles. A quota
status therefore consumes no registry capacity, requires no foreign pointer,
adds no unsafe block, cannot be confused with an owned value, and remains
correlated with its producing call. The wrapper can project the fixed status to
a structured host exception; the raw C ABI still receives a machine-readable
code rather than prose.

This is an out-of-band scalar mechanism, not a sentinel outcome. Fabricating a
tagged value cannot make outcome-copy or release operations succeed because the
tagged range is excluded from the handle namespace.

## Comparison

| Property | Structured sentinel | Status + out pointer | Tagged scalar |
| --- | --- | --- | --- |
| Needs ordinary registry capacity | no | no | no |
| Preserves normal handle release semantics | no | yes | yes |
| Requires special outcome-operation behavior | yes | no | no |
| Adds foreign writable-pointer surface | no | yes | no |
| Correlated under concurrency | yes | yes | yes |
| Carries dynamic/request-specific detail without storage | no | no | no |
| Raw ABI is machine-readable | yes | yes | yes |
| Requires versioned ABI contract | yes | yes | yes |

Quota exhaustion is process-policy admission failure, so fixed code/category
information is sufficient; it must not masquerade as an invocation semantic
failure. Hosts may add current-limit context from their configured policy, but
the native status must not disclose unrelated registry membership or tenant
activity.

## Disposition

The tagged scalar admission result is the nominated exhaustion-delivery shape.
The structured sentinel is rejected as the leading candidate because it breaks
ordinary handle ownership semantics. The output-pointer formulation remains a
fallback if a future ABI needs more return data than a bounded status code.

No tag, code table, quota, threshold, public ABI, or managed exception contract
is admitted. If AR-0017 later selects a quota, an ADR revision or superseding
decision must define the encoded value space, atomic admission point, panic and
quarantine behavior, wrapper mapping, race tests, and exhaustion recovery.

## Required verification after selection

- Prove status and valid-handle ranges are disjoint through handle exhaustion.
- Prove concurrent admission returns either an owned handle or one complete
  status with no partial insertion.
- Prove tagged values are rejected by every outcome inspection/release export.
- Prove releasing an ordinary handle immediately restores capacity.
- Prove exhaustion never evicts or changes an already valid handle.
- Prove managed wrappers never construct `SafeHandle` instances for statuses.
- Preserve bounded structured outcomes for failures that occur after successful
  admission.
