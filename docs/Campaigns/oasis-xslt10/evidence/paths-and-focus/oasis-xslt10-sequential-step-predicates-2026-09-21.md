# OASIS XSLT 1.0 Sequential Step Predicates

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can an ordinary location-path step apply a typed boolean predicate after a
positional predicate while giving the later predicate the position-filtered
focus required by XPath 1.0?

## Method

- Retain positional, context-name, and typed boolean predicates in lexical
  order on the existing private path step.
- Apply each predicate to the sequence produced by the preceding predicate.
- Supply the later boolean predicate with the reduced sequence's position and
  size, and evaluate it through the existing charged and cancellable boolean
  predicate evaluator.
- Add a focused path regression and an end-to-end XSLT 1.0 golden transform
  for `book[position() < 4][title != 'Book 1']`.
- Recheck the unchanged Microsoft `BVTs_bvt099` case that exposed the shape.

## Result

The focused evaluator and complete transform both select `Book 2` and
`Book 3`, proving that the second predicate observes the focus left by the
first predicate rather than the original candidate sequence.

The unchanged corpus case no longer stops at the general location-path parser.
It now reaches the deliberately narrower template-match-pattern boundary and
reports structured `FXST1005`, because the same predicate form occurs in a
match pattern rather than an ordinary selection. The conserved corpus counters
therefore do not change in this tranche.

## Boundaries

- This extends ordinary selection paths only. It does not admit the same form
  into XSLT 1.0 template match patterns.
- Predicate order is semantic and is not fused or reordered.
- The compiled predicate stores no invocation focus or mutable state.
- Existing node traversal charging, boolean short-circuiting, and cancellation
  observation remain in force.

## Reproduction

```powershell
cargo test -p fastxslt --all-features trailing_boolean_predicate_observes_the_position_filtered_focus
cargo test -p fastxslt --all-features xslt10_sequential_position_and_child_value_predicates_preserve_order
./scripts/measure-oasis-xslt10.ps1
```
