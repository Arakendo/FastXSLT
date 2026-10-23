# OASIS XSLT 1.0 Variable-String Template Arguments

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can `string($variable)` be admitted as a selected XSLT 1.0 template argument
by reusing the existing controlled compatibility conversion rather than adding
a template-call-specific value path?

## Method

- Compile an exact `string($variable)` selected argument to a typed private
  argument plan after resolving the variable QName.
- Evaluate it through the shared XSLT 1.0 variable-string conversion, covering
  atomic values, atomic sequences, source-node sequences, empty sequences, and
  temporary trees.
- Return the converted value as an invocation-owned atomic string parameter.
- Add a focused global-temporary-tree-to-template-argument regression.
- Run the hash-verified 3,173-case OASIS CD04 measurement and trace both
  unchanged Lotus `variable65` cases.

## Result

The focused regression produces `<out>Ax</out>`. Both unchanged OASIS
`variable65` cases now match their expected XML results.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,097 | 2,099 | +2 |
| Initialization failures | 1,038 | 1,036 | -2 |
| Executed successfully | 2,010 | 2,012 | +2 |
| Execution failures | 87 | 87 | 0 |
| Exact XML-semantic matches | 1,879 | 1,881 | +2 |
| XML comparison mismatches | 55 | 55 | 0 |

The exact compatibility lower bound is now `1,881 / 3,173 = 59.28%`. The
`FXXP1011` initialization frontier falls from 21 to 19 cases. This is local
compatibility evidence against the hash-verified, non-redistributed OASIS CD04
archive; it is not a broad conformance claim.

## Boundaries

- Only the exact direct-variable `string($variable)` form is newly admitted.
- QName resolution remains compile-time and uses the ordinary variable binding
  identity.
- Conversion remains controlled and invocation-local; no referenced tree or
  sequence is mutated.
- General function calls and arbitrary template-argument expressions remain
  explicit boundaries.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_template_argument_converts_a_temporary_tree_variable_to_string
./scripts/measure-oasis-xslt10.ps1 -TraceCase variable_variable65
./scripts/verify.ps1
```
