# OASIS XSLT 1.0 Variable Node-Set, Union, and Grouped Path

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can the shared XSLT 1.0 path support the node-set operations that follow a
derived global variable in unchanged Microsoft `BVTs_bvt092`, without adding a
second XPath evaluator or weakening bounded execution?

## Method

- Compile `path[$variable = relative/path]` into a typed selection whose
  comparison uses XPath 1.0 existential node-set string-value equality.
- Compile a bounded parenthesized union of source-node variables followed by a
  literal, `last()`, or `last() - N` positional predicate.
- Restore source document order and remove duplicate node identities before
  applying the union position.
- Extend the existing source-variable position representation with checked
  `last() - N` selection for both value and sequence consumers.
- Compile `$left and $right` in XSLT 1.0 value context into typed, short-circuit
  effective-boolean-value operations.
- Admit a redundant outer parenthesis pair around a local variable's location
  path before the existing typed path/cast distinction, without treating the
  grouped path as a cast.
- Keep source nodes in invocation-owned state and charge comparison, path,
  ordering, and boolean work through the existing invocation control.

## Result

Focused complete transforms prove existential node-set comparison, union
document ordering, duplicate removal, literal/last-relative positions,
node-set boolean conjunction, and a parenthesized absolute local-variable
path. The unchanged `BVTs_bvt092` case now initializes, executes, and matches
its expected XML semantics.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,126 | 2,127 | +1 |
| Initialization failures | 1,009 | 1,008 | -1 |
| Executed successfully | 2,037 | 2,038 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,906 | 1,907 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now
`1,907 / 3,173 = 60.10%`. This is local compatibility evidence against a
non-redistributed archival suite, not a broad conformance claim.

## Boundaries

- Node-set comparison is limited to the typed variable-versus-relative-path
  predicate shape; general XPath comparison composition remains unclaimed.
- The positional union accepts at most eight variable-only operands and does
  not admit arbitrary expressions inside the union.
- `last() - N` requires a positive integer literal and uses checked indexing;
  an out-of-range position selects the empty sequence.
- Boolean conjunction is limited to two source/atomic/temporary variable
  effective-boolean values and preserves short-circuit evaluation.
- Grouping admits one non-empty outer parenthesis pair only when its contents
  already compile as the existing typed location path; general parenthesized
  XPath expressions remain outside this slice.
- No source node, invocation variable, or ordered union result is retained in
  compiled or cross-invocation state.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_node_set_variable_comparison_filters_source_paths
cargo test -p fastxslt --all-features xslt10_variable_union_positions_restore_document_order_and_remove_duplicates
cargo test -p fastxslt --all-features xslt10_source_node_variables_support_literal_and_last_positions
cargo test -p fastxslt --all-features parenthesized_absolute_paths_bind_local_source_node_variables
./scripts/measure-oasis-xslt10.ps1 -TraceCase bvt092
./scripts/verify.ps1
```
