# OASIS XSLT 1.0 Global Number Path and Variable Division

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can source-dependent global `number(path)` and variable-only division preserve
XPath 1.0 conversion and non-finite results without weakening the modern
numeric path or retaining invocation data in compiled state?

## Method

- Compile an untyped XSLT 1.0 global `number(path)` as one source-dependent
  typed plan.
- For each invocation, evaluate the path from that invocation's principal
  document, take the first node in document order, derive its controlled string
  value, and store a typed double lexical in the invocation globals.
- Select direct-variable division, optionally wrapped by `string()`, before the
  exact-rational source-path evaluator and reuse the existing compatibility
  conversion and IEEE lexical operation.
- Add focused regressions for first-node global conversion and positive,
  negative, and zero divided by zero.
- Run unchanged Microsoft `Miscellaneous__84362` from the hash-verified local
  OASIS XSLT 1.0 CD04 archive.

## Result

The unchanged case exactly produces `9`, `NaN`, `Infinity`, `-Infinity`, and
`NaN` in its expected locations.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,104 | 2,105 | +1 |
| Initialization failures | 1,031 | 1,030 | -1 |
| Executed successfully | 2,017 | 2,018 | +1 |
| Execution failures | 87 | 87 | 0 |
| Exact XML-semantic matches | 1,886 | 1,887 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,887 / 3,173 = 59.47%`.
This is compatibility evidence against a hash-verified, non-redistributed
archive; it is not a broad conformance claim.

## Boundaries

- The global operation is available only in compile-selected XSLT 1.0 mode.
- It retains a location path, not source nodes or a computed value, in compiled
  state.
- Variable division admits exactly two direct variable operands, with an
  optional outer `string()`.
- General floating-point source-path arithmetic, promotion, and modern
  multi-node `number()` semantics are unchanged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_number_uses_first_node_conversion
cargo test -p fastxslt --all-features xslt10_variable_division_preserves_non_finite_results
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/Miscellaneous__84362
./scripts/verify.ps1
```
