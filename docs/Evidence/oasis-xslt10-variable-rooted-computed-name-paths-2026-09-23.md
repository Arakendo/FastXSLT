# OASIS XSLT 1.0 Variable-Rooted Computed-Name Paths

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can computed element and attribute names use a path rooted at a source-node
variable while preserving the XSLT 1.0 rule that a constructed result-tree
fragment is not navigable as a node-set?

## Method

- Compile an XSLT 1.0 name AVT containing one admitted `$variable/path` into a
  typed variable-rooted name plan.
- Reuse the ordinary charged location-path evaluator when the invocation
  variable owns principal-source nodes.
- Apply first-node string conversion, document-order normalization, and the
  existing computed-name QName and namespace validation.
- Reject atomic, absent, or temporary-tree values with `XPTY0019` rather than
  navigating a constructed tree through an accidental extension.
- Compare a focused legal source-node case and the unchanged OASIS
  `Microsoft/Variables__91490#1` expected-error case.

## Result

A source-node variable now supplies both an element name and an attribute name
through the shared computed-constructor path. The unchanged Microsoft case
initializes, then reports `XPTY0019` because `$foo` contains constructed
temporary-tree content. It moves from an unsupported initialization frontier
to an observed expected execution error.

The conserved 3,173-case sweep now initializes 2,210 cases. Successful
execution remains 2,156; execution failures rise to 54 because the newly
initialized expected-error case reaches its required runtime failure. Expected
errors observed during execution rise from 28 to 29, and unexpected successful
expected-error cases fall from 7 to 6. Exact expected-result matches remain
2,008 / 3,173 (63.32%), with 64 visible mismatches and 75 comparator-unsupported
cases.

## Boundaries

- The plan is selected only for XSLT 1.0 compatibility and contains no runtime
  stylesheet-version branch.
- It accepts only the existing bounded variable-rooted location-path grammar.
- It does not provide an implicit `node-set()` extension for result-tree
  fragments.
- Variable lookup, path work, and string-value work remain invocation-owned
  and charged.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_source_variable_paths_supply_computed_names_but_temporary_trees_do_not
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Variables__91490#1'
./scripts/verify.ps1
```
