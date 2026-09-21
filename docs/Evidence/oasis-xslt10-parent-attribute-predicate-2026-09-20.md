# OASIS XSLT 1.0 Parent-Attribute Predicate

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can a final path predicate compare an immediate parent's unqualified attribute
with a string literal without flattening parent focus or introducing general
relative-expression evaluation?

## Method

- Compile the exact `../@name = 'literal'` and inequality forms as a typed
  predicate.
- At runtime, charge the parent visit, inspect only that parent's attributes,
  and reuse existing XPath node-set/string equality or inequality behavior.
- Add a focused descendant-selection regression.
- Run unchanged Lotus `copy42` from the hash-verified local OASIS XSLT 1.0
  CD04 archive.

## Result

The unchanged case initializes, executes, sorts the selected nodes numerically,
and exactly matches its expected XML.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,111 | 2,112 | +1 |
| Initialization failures | 1,024 | 1,023 | -1 |
| Executed successfully | 2,022 | 2,023 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,891 | 1,892 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,892 / 3,173 = 59.63%`.
The generic `FXXP1001` initialization frontier falls from 35 to 34 cases.
This is compatibility evidence against a hash-verified, non-redistributed
archive; it is not a broad conformance claim.

## Boundaries

- The parent step is exactly `..`, followed by one unqualified attribute.
- The other operand is one XPath string literal and the operator is `=` or
  `!=`.
- General parent paths, qualified attributes, dynamic operands, and composed
  parent predicates remain unsupported.
- The typed plan retains only lexical names and values; all node access remains
  invocation-owned, cancellable, and work charged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features path_boolean_predicate_compares_parent_attributes_with_string_literals
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/copy_copy42
./scripts/verify.ps1
```
