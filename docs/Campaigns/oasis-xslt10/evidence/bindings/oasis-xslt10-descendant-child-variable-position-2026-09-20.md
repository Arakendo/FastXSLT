# OASIS XSLT 1.0 Descendant-Child Variable Position

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can FastXSLT execute the exact `.//NCName[$position]` XSLT 1.0 value form
without incorrectly applying the numeric predicate to the flattened descendant
result?

## Method

- Compile the exact descendant-or-self abbreviation followed by one named child
  step and one direct numeric-variable predicate into a typed compatibility
  plan.
- Evaluate the ordinary charged location path, then count matching child
  positions independently for each immediate parent. This preserves the XPath
  expansion `descendant-or-self::node()/child::NCName[$position]` rather than
  treating the expression as `(descendant::NCName)[$position]`.
- Convert the variable through the shared XSLT 1.0 variable-string and XPath
  numeric conversion owners.
- Preserve first-node document-order string conversion, cancellation
  observation, source provenance, and prepared-input ownership.
- Add a focused regression where a flattened second node differs from the
  first parent's second matching child, then run unchanged Lotus
  `position_position97` from the hash-verified local OASIS XSLT 1.0 CD04
  archive.

## Result

The focused regression returns the per-parent result, and the unchanged corpus
case exactly matches its expected XML result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,112 | 2,113 | +1 |
| Initialization failures | 1,023 | 1,022 | -1 |
| Executed successfully | 2,023 | 2,024 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,892 | 1,893 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,893 / 3,173 = 59.66%`.
The generic `FXXP1001` initialization frontier falls from 34 to 33 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- The admitted expression is exactly `.//NCName[$variable]` with ASCII NCNames.
- The variable must convert to a finite positive integer; all other numeric
  values select no node.
- The per-parent counters are invocation-owned scratch bounded by the selected
  source nodes; no source or invocation state enters the compiled plan.
- General multi-step variable predicates, arbitrary predicate expressions,
  grouped descendant filters, and sequence filtering remain unsupported.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_descendant_child_variable_positions_preserve_per_parent_focus
./scripts/measure-oasis-xslt10.ps1 -TraceCase position_position97
./scripts/verify.ps1
```
