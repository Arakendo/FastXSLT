# Path Location Owner Decomposition Checkpoint

Date: 2026-08-28

## Authority and purpose

This checkpoint executes the bounded follow-up from the
[path location-step cohesion review](path-location-step-cohesion-review-2026-08-28.md)
under [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md).
It is a behavior-preserving private decomposition and terminology correction,
not a new XPath semantic slice.

## Mechanical changes

- `ChildPath` became `LocationPath`.
- `parse_child_path` became `parse_location_path`.
- `evaluate_child_path` and its controlled form became
  `evaluate_location_path` and `evaluate_location_path_controlled`.
- The XSLT semantic variants carrying that private representation became
  `LocationPath` variants.
- Compiler, runtime, and sibling XPath callers were updated mechanically.
- The embedded `#[cfg(test)]` body moved unchanged to the private sibling
  `path_experiment_tests.rs` module.
- A stale child-only grammar diagnostic and compiler-test name now use
  location-path terminology.

No public API exists for these types or functions. No parser rule, typed step,
node selection, diagnostic category/location, work charge, XDM representation,
or host boundary changed.

## Responsibility result

| Unit | Before | After | Responsibility |
| --- | ---: | ---: | --- |
| `path_experiment.rs` | 1,092 lines | 527 lines | private location-path grammar, typed lowering, evaluation, diagnostics, and work charging |
| `path_experiment_tests.rs` | absent | 561 lines | path grammar/navigation/predicate/order/diagnostic/work-charge invariants |

The production owner has a one-way test child only under `cfg(test)`. The test
module imports the private subject through `super`; production code does not
depend on the test module, no shared context was introduced, and no runtime
indirection or allocation was added.

## Conservation checks

- All 18 focused path invariants pass under their existing module-qualified
  test identities.
- Complete QT3 `Axes001` through `Axes011` still produce 37 native passes.
- The full workspace gate verifies formatting, strict Clippy, all features,
  documentation, local links, unsafe-surface policy, and pinned corpus
  inventories.

## Disposition

The named checkpoint is complete. Later reverse-axis or kind-test work may now
proceed incrementally. A parser/evaluator production split remains unselected;
it still requires evidence of independent responsibility or coupling pressure.

The later [path test-owner cohesion checkpoint](path-test-owner-cohesion-checkpoint-2026-08-28.md)
separates syntax/diagnostic invariants from evaluation invariants after the
test owner approached the next ADR-0004 size threshold. Production parsing and
evaluation remain together.
