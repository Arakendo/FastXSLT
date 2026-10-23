# OASIS XSLT 1.0 Intermediate Ancestor-Count Predicate

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can the shared location-path representation retain a typed boolean predicate
on an intermediate step, and can that predicate compare the count of element
ancestors before later parent-axis navigation?

## Method

- Give each compiled path step an optional private typed boolean predicate,
  evaluated after its ordinary axis/name test and before its positional
  filters and subsequent navigation.
- Compile exact `count(ancestor::*) comparison nonnegative-integer` predicates
  with all six ordinary comparison operators.
- Traverse parents through the immutable XDM, count only element ancestors,
  charge every inspected ancestor and the final comparison, and preserve
  cancellation observation.
- Add a focused compound descendant/predicate/parent-axis regression and run
  unchanged Lotus `axes_axes84` from the hash-verified local OASIS XSLT 1.0
  CD04 archive.

## Result

The focused path `//*[count(ancestor::*) >= 2]/../parent::*` selects the five
distinct grandparents in document order. The unchanged corpus case
initializes, executes, and exactly matches its expected XML result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,121 | 2,122 | +1 |
| Initialization failures | 1,014 | 1,013 | -1 |
| Executed successfully | 2,032 | 2,033 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,901 | 1,902 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,902 / 3,173 = 59.94%`.
The generic `FXXP1001` initialization frontier falls from 25 to 24 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- The new intermediate predicate slot reuses the existing typed boolean-plan
  owner; it does not expose a public AST or a general predicate grammar.
- This count operand is exactly the `ancestor::*` element axis. Named tests,
  other axes, dynamic operands, nested predicates, and arithmetic remain
  unsupported here.
- Step results still normalize node identity and document order before the
  following step; no invocation state enters the compiled path.

## Reproduction

```powershell
cargo test -p fastxslt --all-features path_boolean_predicates_compare_ancestor_element_counts
./scripts/measure-oasis-xslt10.ps1 -TraceCase axes_axes84
./scripts/verify.ps1
```
