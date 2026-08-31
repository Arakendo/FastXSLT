# Deep-equal source-unit cohesion review

## Status

| Field              | Value                                                                               |
| ------------------ | ----------------------------------------------------------------------------------- |
| Date               | 2026-08-27                                                                          |
| Governing decision | [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md) |
| Reviewed unit      | `crates/fastxslt/src/xpath/deep_equal_experiment.rs`                                |
| Physical size      | 1,054 lines before extraction; 661-line parent and 505-line child after extraction  |
| Disposition        | Completed: private atomic owner extracted                                           |

## Trigger

The unit crossed the 1,000-line inspection threshold during a substantive
standards change and now has a named responsibility seam. It owns both:

- node-selection and node deep-equality behavior over the private XDM tree; and
- atomic lexical construction, ordered atomic sequences, numeric promotion,
  calendar validation, and atomic deep-equality behavior.

Crossing the size threshold while satisfying that responsibility trigger
requires this retained review under ADR-0004. The line count is a signal; the
independently testable node and atomic subjects justify decomposition.

## Conservation checkpoint

The semantic checkpoint executes:

- the complete two-case XSLT30 `fn/deep-equal` node denominator;
- the admitted complete five-case QT3 typed numeric groups;
- the complete 31-case QT3 `fn-deep-equal-mix-args-*` group with exact upstream
  and overlay denominator assertions;
- focused value-space, ordered-sequence, promotion, NaN, boolean, and calendar
  controls; and
- the ordinary workspace formatting, Clippy, test, documentation, link,
  corpus-integrity, and reviewed-unsafe-surface gates.

The semantic work is checkpointed before extraction so a later structural
change cannot quietly repair or reinterpret it.

## Selected ownership seam

The private `deep_equal_atomic.rs` child owns:

- the private atomic value and exact-decimal representations;
- depth-aware parsing of the admitted parenthesized atomic sequences;
- admitted constructor lexical validation;
- numeric promotion and atomic comparison; and
- focused atomic semantic tests.

Its inputs are expression slices. Its outputs are private parsed atomic
sequences or an explicit non-match/parse result consumed by the parent. It does
not own XDM documents, node navigation, invocation control, diagnostics,
stylesheet compilation, serialization, host resources, or public API.

The existing deep-equal parent retains:

- recognition and location of the function call;
- selection among node, scalar, and atomic operand paths;
- node-selection parsing and node comparison;
- invocation work charging and missing-context failures; and
- composition of unsupported diagnostics.

Dependency direction is one way: the parent calls the private atomic child.
The child must not call back into the parent or receive a broad shared context.

## Coupling result

The extraction removes constructor and atomic comparison growth from the node
owner. Future node kinds can evolve without touching float, boolean, or calendar
logic; future atomic types can evolve without importing XDM navigation. A
505-line child now has focused tests requiring no `Document` or
`InvocationControl`, while the 661-line parent continues to own work accounting
at the comparison boundary. The child receives no shared context and has no
dependency on the parent, XDM, execution control, diagnostics, or resource
authority. Its narrow surface consists of private parse results, sequence
length/item comparison, and the top-level delimiter helper used by the parent.

## Contract and risk consequences

- No public Rust API, crate boundary, dependency, host ABI, or resource
  authority changes.
- No unsafe code is introduced or moved.
- Atomic values remain private representation, not an inspection contract.
- The extraction must preserve exact diagnostics, source locations, work
  charges, corpus dispositions, and early mismatch behavior.
- The extraction should be mechanically attributable. Semantic fixes discovered
  during it require a separate checkpoint after conservation is restored.

## Disposition

The private atomic extraction is complete. The parent retains function
recognition, node semantics, diagnostics, source locations, and invocation work
charging. The child owns the admitted atomic representations, constructor
lexicals, sequence parsing, promotions, and item comparison.

The conservation checkpoint passed after extraction: the focused deep-equal
selection ran 36 tests, including the complete QT3 and XSLT30 denominators and
five new representation-local controls. Existing boundary tests continue to
verify exact early-mismatch charging through `InvocationControl`. No public
API, host ABI, crate boundary, dependency, resource authority, unsafe surface,
or claimed standards behavior changed.

Reopen the review if the child still consumes most parent state, work charging
must move across the seam, diagnostics lose provenance, performance changes
materially, another independent responsibility appears, or either resulting
unit again crosses an ADR-0004 threshold.

## Follow-up evolution

The next K-family corpus slices validated the seam. Atomic constructor and
three-item sequence corrections remained in the atomic child, now 516 lines.
Static arity and explicit codepoint-collation recognition remained in the
711-line parent. Outer `not(...)` and boolean-equality composition became a
separate 96-line owner above the function rather than adding a third
responsibility to either existing unit. Dependency direction remains one way:
composition calls the function parent, which calls the atomic child. No unit
has reached a reopening threshold.
