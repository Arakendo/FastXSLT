# Peer Review: Activated Execution Paths and Unsafe Optimization

Date: 2026-08-26

The project owner and Monday discussed the performance headroom available when
compiled stylesheets activate only the semantic machinery they require.

The useful architectural direction is compile-time semantic discovery followed
by a specialized reusable execution plan. Simple transforms should not pay
runtime branching or initialization costs for unrelated maps, arrays, packages,
schema typing, streaming metadata, or other unneeded language facilities.

The review also retained ADR-0003's ordering for future optimization:

```text
safe semantic reference
    -> proven invariants
    -> optimized safe path
    -> optional narrow unsafe specialization
    -> differential verification
```

Compact IDs/arenas, interned names, pre-resolved name tests, template indexes,
static context, specialized values, scratch reuse, constant folding, and opcode
specialization remain substantial safe optimization candidates. Unsafe hot-path
work is justified only by profiling and a separate exact exception; ADR-0008's
native FFI permission does not authorize any of these engine optimizations.

This is design pressure, not evidence that a particular plan representation,
opcode engine, optimizer, unsafe fast path, or performance gain exists.
