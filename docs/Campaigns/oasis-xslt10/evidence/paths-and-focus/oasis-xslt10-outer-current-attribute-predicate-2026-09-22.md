# OASIS XSLT 1.0 Outer-Current Attribute Predicate

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Can XSLT 1.0 location paths compare a candidate node's attribute with an
attribute on the instruction's outer `current()` node without admitting
`current()` into the modern XPath parser or adding a second evaluator?

## Method

- Normalize only the bounded form `@candidate = current()/@outer` while parsing
  an XSLT 1.0 expression.
- Lower that form into a typed path predicate which retains both attribute
  names.
- Evaluate the candidate attribute against the original instruction context,
  preserving work charges, document order, cancellation, and the ordinary
  location-path evaluator.
- Route XSLT 1.0 `xsl:sort` and `xsl:copy-of` paths through the compatibility
  parser; keep modern XPath on the unchanged general parser.
- Exercise relative, absolute, and descendant origins in focused tests, then
  rerun the unchanged 3,173-case OASIS catalog.

## Result

Three formerly uninitialized cases now execute and compare exactly:

- `Microsoft/Sorting_SortExprWithCurrentInsideForEach#1`;
- `Microsoft/Sorting_SortExprWithCurrentInApplyTemplates#1`;
- `Lotus/select_select03#1`.

The conserved totals are 2,143 initialized cases, 992 initialization failures,
2,083 successful executions, 60 execution failures, 1,932 exact XML-semantic
matches, and 61 mismatches. The strict compatibility lower bound is
`1,932 / 3,173 = 60.89%`.

## Boundaries

- This is one typed XSLT 1.0 comparison form, not a general implementation of
  every `current()` expression.
- `current()` remains unavailable to the modern XPath location-path parser.
- Candidate and outer attributes are unprefixed in this slice; QName and
  namespace expansion require separate evidence.
- No compiled or prepared state retains invocation-specific node identity.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_path_predicate_compares_candidate_and_outer_context_attributes
cargo test -p fastxslt --all-features xslt10_outer_current_attribute_predicates_compose_with_sort_and_copy_of
./scripts/measure-oasis-xslt10.ps1 -TraceCase SortExprWithCurrent
./scripts/measure-oasis-xslt10.ps1 -TraceCase select_select03
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
