# Native Boundary Diagnostic-Test Decomposition

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Governing decision | ADR-0004 |
| Native boundary at inspection | 1,061 physical lines |
| Native boundary after extraction | 989 physical lines |
| Diagnostic companion | 107 physical lines |
| Unsafe surface | Unchanged: 2 blocks, 15 exports, 17 allowances |
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

## Conservation

The extraction changes no export, symbol, argument, outcome encoding, pointer
operation, handle lifecycle, quarantine rule, safe engine facade, or unsafe
contract. The verification script's exact unsafe counts remain unchanged. Both
moved tests pass under their original names and still exercise the actual
native encode/copy/release path.
