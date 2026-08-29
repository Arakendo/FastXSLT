# Path Location-Step Cohesion Review

Date: 2026-08-28

## Trigger and scope

| Item | Observation |
| --- | --- |
| Governing decision | [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md) |
| Source unit | `crates/fastxslt/src/xpath/path_experiment.rs` |
| Before attribute-axis tranche | 975 physical lines |
| After focused implementation and controls | 1,092 physical lines |
| Size treatment | Inspect cohesion during this substantive modification |

The review covers only the private admitted location-path subset. It does not
select a public XPath AST, general parser architecture, alternate evaluator, or
crate boundary.

## Current responsibilities

The production portion owns one connected responsibility:

1. classify the admitted relative location-path grammar;
2. lower admitted syntax to typed steps and private predicates;
3. evaluate those steps against owned XDM in document/axis order;
4. retain structured invalid-versus-unsupported locations; and
5. charge invocation-local XPath navigation work.

The parser constructs the exact private variants consumed by the evaluator.
The evaluator does not reparse strings, duplicate abbreviated syntax, acquire
resources, or expose representation details to a host. Child and attribute
steps share the same loop, position filtering, failure type, and work-control
contract.

The source also contains a large `#[cfg(test)]` body organized around path
grammar, navigation, ordering, predicates, diagnostic classification, and
exact work charges. It is semantically related but independently navigable.

## Candidate dispositions

### Retain production parsing and evaluation together now

Splitting syntax from evaluation during the `Axes007`–`Axes011` semantic change
would require moving the typed path representation and failure contract to a
third owner or creating parent pass-through access. There is only one evaluator
and no measured compile-time or change-coupling pressure demonstrating that
such a split reduces responsibility coupling. The production responsibility is
therefore retained for this completed tranche.

### Extract the invariant test owner at the next checkpoint

The embedded test body consumes a narrow set of private parser/evaluator types
through `super` and does not provide production behavior. Moving it to a named
private child test module can reduce navigation burden without creating a
production dependency or weakening assertions. Per ADR-0004's checkpoint rule,
that mechanical extraction follows this semantic commit rather than being
mixed into it.

### Correct stale child-only terminology separately

`ChildPath` and `parse_child_path` predate attribute-axis support. They are
private but now communicate a narrower subject than they own. Renaming them to
location-path terminology touches compiler, runtime, XSLT semantic, and XPath
experiment callers without changing behavior. That conservation-sensitive
mechanical change belongs with the next checkpoint, not inside the standards
tranche.

## Disposition

Retain the cohesive production owner through `Axes007`–`Axes011`. Before adding
another axis family:

1. extract the invariant tests to a private path-owned test module;
2. rename child-only private path terminology to location-path terminology;
3. rerun exact focused, workspace, corpus-inventory, diagnostic, and work-charge
   gates; and
4. reassess whether later reverse-axis or kind-test pressure demonstrates a
   real syntax/evaluation ownership seam.

No public API, additional axis, alternative backend, unsafe path, cache, or
representation guarantee is admitted by this review.

## Follow-up result

The required extraction and terminology correction are complete in the
[path location owner decomposition checkpoint](path-location-owner-decomposition-checkpoint-2026-08-28.md).
The production owner is 527 lines and its private invariant-test child is 561
lines, with focused and workspace conservation gates retained.
