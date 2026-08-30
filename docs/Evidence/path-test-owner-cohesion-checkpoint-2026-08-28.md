# Path Test-Owner Cohesion Checkpoint

Date: 2026-08-28

## Trigger

The private `path_experiment_tests.rs` owner reached 992 physical lines after
the selected QT3 `Axes080` through `Axes084` attribute-composition tranche. A
further substantive test addition would cross ADR-0004's 1,001-line cohesion
inspection threshold.

This checkpoint is a behavior-preserving private test decomposition. It does
not admit new XPath grammar, semantics, diagnostics, representation, or public
API.

## Responsibility inspection

The owner contained two independently navigable invariant families:

1. grammar lowering, abbreviated-versus-explicit typed-step equivalence, ASCII
   name-test classification, and invalid-versus-unsupported diagnostics; and
2. XDM navigation, axis ordering, identity deduplication, predicates,
   positions, and exact work charges.

These families share the private path types but not test fixtures or mutable
context. Separating them creates one-way test-only dependencies on the same
production subject and introduces no sibling coupling.

## Mechanical change

| Unit                       |    Before |     After | Responsibility                                                                          |
| -------------------------- | --------: | --------: | --------------------------------------------------------------------------------------- |
| `path_experiment_tests.rs` | 992 lines | 891 lines | path evaluation, XDM navigation/order, predicates, identity, and work-charge invariants |
| `path_syntax_tests.rs`     |    absent | 112 lines | path grammar lowering and diagnostic-classification invariants                          |
| `path_experiment.rs`       | 757 lines | 761 lines | unchanged production owner plus two private `cfg(test)` module declarations             |

Four tests moved without weakening assertions. Their module-qualified test
names now identify the syntax responsibility explicitly. Twenty-two evaluation
tests remain in the evaluation owner. No helper, production type, runtime call,
allocation, visibility, or crate boundary was introduced.

## Conservation

- all 4 syntax-focused tests pass;
- all 22 evaluation-focused tests pass;
- the metadata-driven QT3 axis seam remains at 181 passing selected cases;
- the complete workspace verification gate remains required before commit.

## Disposition

Accept the private test split. Retain production parsing and evaluation in the
same cohesive owner: there remains one parser, one evaluator, and no evidence
that moving their shared typed representation would reduce coupling.
