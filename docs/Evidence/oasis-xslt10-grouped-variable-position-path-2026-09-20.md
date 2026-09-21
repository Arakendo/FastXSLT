# OASIS XSLT 1.0 Grouped Variable-Position Path

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can FastXSLT execute the bounded `(path)[$position]/suffix` XSLT 1.0 value form
without flattening its predicate into a semantically different path step?

## Method

- Compile the grouped selection, canonical variable name, and suffix as one
  typed compatibility plan.
- Evaluate the grouped path in document order, convert the variable using the
  shared XSLT 1.0 variable-string rules, select that numeric position, and only
  then evaluate the suffix from the selected node.
- Retain existing path work charging, cancellation observation, source
  provenance, first-node string conversion, and prepared-input ownership.
- Add a focused runtime regression and run the unchanged Microsoft
  `Variables__78116` and `Variables__78354` cases from the hash-verified local
  OASIS XSLT 1.0 CD04 archive.

## Result

Both unchanged corpus cases now exactly match their expected XML results.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,102 | 2,104 | +2 |
| Initialization failures | 1,033 | 1,031 | -2 |
| Executed successfully | 2,015 | 2,017 | +2 |
| Execution failures | 87 | 87 | 0 |
| Exact XML-semantic matches | 1,884 | 1,886 | +2 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,886 / 3,173 = 59.44%`.
The generic `FXXP1001` initialization frontier falls from 44 to 42 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- The predicate is exactly one direct numeric variable reference.
- The expression must contain one grouped location path and a non-empty
  location-path suffix.
- General parenthesized filters, arbitrary predicates, sequence expressions,
  and grouped-path sorting remain unsupported.
- The plan retains no source nodes or invocation values in compiled state.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_grouped_path_filters_by_variable_before_navigating_suffix
./scripts/measure-oasis-xslt10.ps1 -TraceCase Variables__78116
./scripts/measure-oasis-xslt10.ps1 -TraceCase Variables__78354
./scripts/verify.ps1
```
