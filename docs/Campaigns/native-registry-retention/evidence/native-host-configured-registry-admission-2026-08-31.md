# Native host-configured registry admission — 2026-08-31

## Scope

This tranche implements ADR-0016 in the unpublished native .NET workbench. It
does not promote the ABI, select native execution as a default, or choose
production quota values.

## Implemented mechanism

- The native ABI advances to experimental version 3 and adds one scalar-only
  policy configuration export.
- The host supplies engine, active-control, and outcome counts; exact aggregate
  outcome payload bytes; aggregate known prepared-engine capacity; and total
  accounted bytes.
- Configuration is process-wide and one-shot. Repeating exactly the same policy
  is idempotent; a conflicting policy is rejected even after handles drain.
- Handle-producing operations reject an omitted policy through a tagged scalar
  without compiling, preparing, or allocating an outcome.
- Every admission checks family count before family byte/capacity limits and
  total accounted bytes. Engine plus creation-outcome publication remains
  all-or-nothing.
- Outcome and engine release remove their recorded charge in the same safe-Rust
  critical section that removes the handle. No admission path evicts a live
  handle.
- Ordinary handles retain the high bit clear. Quota statuses use the high bit,
  tag version zero, and fixed codes for missing policy plus each exhausted
  dimension. They consume no registry slot and cannot be released as handles.
- The managed adapter recognizes tags before any outcome or `SafeHandle`
  operation and maps them to `FXFFI0101` through `FXFFI0107` with category
  `resource-exhausted`.

The private policy vocabulary and accounting live in their own safe module;
the FFI owner retains handle synchronization and boundary projection.

## Verification

Focused Rust tests prove:

- required, identical, and conflicting one-shot configuration;
- exact count and byte boundaries;
- distinct engine-count, known-capacity, and aggregate-byte statuses;
- immediate capacity recovery after release;
- no publication after rejected engine admission;
- tagged values are absent from registries and cannot be released; and
- exactly one winner when two threads race for the last outcome slot.

A separate managed-process quota smoke configures zero active controls, calls
the real P/Invoke operation, and observes exactly
`FXFFI0103 / resource-exhausted`. The normal release-built ASP.NET smoke then
configures its historical comparison policy explicitly, loads ABI version 3,
and preserves the isolated, native, and Microsoft semantic sentinels.

The unsafe surface remains two blocks. One scalar export and its scoped export
allowance advance the enforced totals to 21 exports and 23 allowances.

## Guarantee boundary

The byte dimensions account exact retained outcome payloads and the private
known prepared-engine-capacity observation. They do not cap compilation or
preparation peak, allocator metadata, CLR objects, managed result copies,
working set, private bytes, or unrelated process state. A host requiring a hard
memory ceiling and forced abandonment reclamation must use an externally
limited isolated worker.

The workbench's explicit unlimited policy is a comparison-host opt-out, not a
FastXSLT production default. Consumer hosts own their values under ADR-0016's
host-owned operational-policy principle.
