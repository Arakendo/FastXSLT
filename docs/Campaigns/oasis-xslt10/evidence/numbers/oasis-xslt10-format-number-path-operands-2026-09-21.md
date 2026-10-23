# OASIS XSLT 1.0 `format-number()` Path Operands

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can XSLT 1.0 `format-number()` obtain its number and picture from child
node-sets without adding formatter-owned navigation or changing modern
formatting semantics?

## Method

- Admit simple child paths only when compiling an XSLT 1.0 expression.
- Store the ordinary typed location paths in the formatting plan.
- At execution, evaluate each path through the existing charged and cancellable
  location-path evaluator, then apply XSLT 1.0 first-node string conversion.
- Keep the empty-picture/non-finite behavior behind the compiled XSLT 1.0
  compatibility flag.
- Accept both XPath 1.0 string delimiters when splitting formatting arguments,
  including commas and apostrophe literals inside a double-quoted picture.
- Run focused lifecycle and formatter tests, then rerun all 3,173 unchanged
  OASIS catalog cases.

## Result

The focused lifecycle formats child-selected `1234.5` and `#,##0.00` as
`1,234.50`. Microsoft `XSLTFunctions__emptyParameters` now executes unchanged
and exactly produces `<test>NaN</test>`. The former
`unsupported/FXXP1009/FXXP1009` initialization frontier is eliminated.

Double-quoted picture parsing also advances Microsoft `BVTs_bvt024` to its
independent `disable-output-escaping="yes"` boundary; it is not counted as a
new execution or pass.

The conserved totals are 2,140 initialized cases, 995 initialization failures,
2,061 successful executions, 79 execution failures, 1,918 exact XML-semantic
matches, and 56 mismatches. The strict compatibility lower bound is
`1,918 / 3,173 = 60.45%`.

## Boundaries

- This does not introduce a formatter-local tree walker. Path work retains the
  shared evaluator's ordering, work charging, cancellation, and source
  semantics.
- Path-operand admission and empty-picture/non-finite behavior are selected by
  XSLT 1.0 compilation; the modern formatting path is unchanged.
- Only the already admitted simple child-path grammar is added here.
- `disable-output-escaping` remains explicitly unsupported at the semantic
  result-tree boundary.

## Reproduction

```powershell
cargo test -p fastxslt --all-features format_number_experiment::tests
cargo test -p fastxslt --all-features xslt10_format_number_converts_first_nodes_selected_by_child_paths
./scripts/measure-oasis-xslt10.ps1 -TraceCase XSLTFunctions__emptyParameters
./scripts/measure-oasis-xslt10.ps1 -TraceCase BVTs_bvt024
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier unsupported/FXXP1009/FXXP1009
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
