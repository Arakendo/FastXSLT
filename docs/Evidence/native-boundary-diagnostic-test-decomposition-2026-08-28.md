# Native Boundary Diagnostic-Test Decomposition

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Governing decision | ADR-0004 |
| Native boundary at inspection | 1,061 physical lines |
| Native boundary after ADR-0011 | 1,086 physical lines |
| Diagnostic companion after ADR-0011 | 176 physical lines |
| Unsafe surface | 2 blocks, 16 exports, 18 allowances |
| Disposition | Inspection trigger discharged by private test-owner extraction |

## Trigger and ownership

Adding real resource-authority envelope cases moved the native boundary into
ADR-0004's 1,001–2,000 inspection band. The production unit still coherently
owns the reviewed native ABI: synchronous pointer copying, handle registries,
panic quarantine, control handles, outcomes, and exports. No new production
responsibility or independently justified ABI owner appeared.

The growing diagnostic-envelope tests were independently navigable and moved
to `diagnostic_tests.rs`. That private companion owns byte-level verification
that engine diagnostic code, category, request, location, span, and detail
survive native outcome encoding. General native lifecycle and control tests
remain beside the ABI behavior they exercise.

ADR-0011 later returned the parent to the inspection band by adding a second
creation operation. That operation has the same native-boundary responsibility,
copy primitive, registry lifecycle, and panic containment as the original
creation symbol. No second owner or independent crate boundary appeared, so the
unit is retained after renewed inspection. The dependency transport tests stay
with the diagnostic companion because they conserve the same seven-field
authority outcomes.

## Conservation

The original extraction changed no export, symbol, argument, outcome encoding,
pointer operation, handle lifecycle, quarantine rule, safe engine facade, or
unsafe contract. ADR-0011 subsequently adds one reviewed export and allowance
but no unsafe block or new pointer operation. The verification script now
enforces the updated exact counts, and all companion tests exercise the actual
native create/encode/copy/release paths.
