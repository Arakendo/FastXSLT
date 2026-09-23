# OASIS XSLT 1.0 Outer-Current Parent/Child Predicate

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can an XSLT 1.0 location-path predicate compare a child of the candidate's
parent with the node returned by `current()` without turning `current()` into
ordinary XPath context or admitting a general expression evaluator?

## Method

- Add one private typed predicate for `../name = current()` and its symmetric
  spelling.
- Select that predicate only through the XSLT 1.0 path entry point; ordinary
  XPath parsing continues to reject `current()`.
- Preserve the instruction's outer context separately from each predicate
  candidate's focus.
- Traverse the candidate parent and its named children through the existing
  charged path evaluator, comparing node string values with XPath 1.0 node-set
  equality semantics for the singular outer node.
- Add focused parser/evaluator and end-to-end runtime regressions.
- Rerun the unchanged 3,173-case OASIS catalog.

## Result

The unchanged `Microsoft/XSLTFunctions__84421#1` case moves from the generic
`FXXP1001` initialization frontier to an exact expected-result match.

The complete sweep initializes 2,196 cases and executes 2,143 successfully.
Exact XML-semantic matches rise from 1,996 to 1,997. Execution failures remain
53, mismatches remain 62, and comparator-unsupported results remain 75. The
strict complete-catalog lower bound is `1,997 / 3,173 = 62.94%`.

## Boundaries

- This is not a general implementation of XPath filter expressions or the
  XSLT `current()` function in arbitrary expressions.
- The ordinary XPath parser does not acquire `current()`.
- Parent traversal, child traversal, comparison work, cancellation, and
  budgets remain charged through the existing invocation control.
- Node identity, source provenance, resource authority, and prepared-input
  ownership are unchanged.
- A broader `current()` expression must earn its own typed representation and
  corpus evidence rather than passing through this normalization.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_path_predicate_compares_a_parent_child_with_the_outer_context
cargo test -p fastxslt --all-features xslt10_outer_current_compares_with_a_candidate_parent_child
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/XSLTFunctions__84421#1'
./scripts/verify.ps1
```
