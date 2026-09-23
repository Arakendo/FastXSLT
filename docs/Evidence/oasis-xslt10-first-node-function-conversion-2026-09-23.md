# OASIS XSLT 1.0 First-Node Function Conversion

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can XSLT 1.0 node-set arguments to `name()`, `local-name()`,
`namespace-uri()`, `string()`, and `normalize-space()` use the first node in
document order without weakening modern zero-or-one cardinality?

## Method

- Select explicit private XSLT 1.0 first-node plans during stylesheet
  compilation.
- Keep the modern plans and their `XPTY0004` cardinality checks unchanged.
- Reuse the charged location-path evaluator, which already returns normalized
  document-order node identities.
- Convert constructed XSLT 1.0 variable content through the existing atomic,
  source-node, and temporary-tree string-value owner before normalization.
- Charge normalization work per inspected character.
- Add a focused cross-version regression covering all five functions, a
  multi-node source selection, and a constructed temporary-tree variable.
- Rerun the unchanged 3,173-case OASIS catalog.

## Result

The four unchanged cases `Lotus/node_node05#1`,
`Lotus/string_string32#1`, `Lotus/string_string85#1`, and
`Lotus/string_string122#1` move from `XPTY0004` execution failures to exact
expected-result matches.

The complete sweep still initializes 2,195 cases. Successful executions rise
from 2,138 to 2,142, execution failures fall from 57 to 53, and exact
XML-semantic matches rise from 1,989 to 1,993. Mismatches remain 65 and
comparator-unsupported results remain 75. The strict complete-catalog lower
bound is `1,993 / 3,173 = 62.81%`.

## Boundaries

- Compatibility is selected at compilation; runtime does not inspect the
  stylesheet version.
- Modern `name()`, `local-name()`, `namespace-uri()`, `string()`, and
  `normalize-space()` plans still reject a sequence containing more than one
  item where their modern signatures require zero-or-one.
- No general XPath 1.0 coercion layer or second evaluator is introduced.
- Temporary-tree conversion remains invocation-owned and does not retain or
  share constructed state beyond the existing variable lifetime.
- Resource authority, parser behavior, source identity, cancellation, and
  budgets are unchanged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_node_set_string_functions_use_the_first_node_in_document_order
cargo test -p fastxslt --all-features node_name_path_rejects_more_than_one_node
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/node_node05#1'
./scripts/verify.ps1
```
