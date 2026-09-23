# OASIS XSLT 1.0 Multiple-Variable Apply Union

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can XSLT 1.0 `xsl:apply-templates` combine multiple source-node variables in
one union without adding an alternate union evaluator or weakening document
order and duplicate semantics?

## Method

- Route every admitted XSLT 1.0 top-level apply-selection union through the
  existing bounded union compiler rather than limiting it to expressions that
  contain `key()`.
- Keep the existing eight-alternative ceiling and the all-key specialization.
- Lower mixed path, key, and source-node-variable members into the existing
  `Xslt10MixedUnion` representation.
- Evaluate each typed member through its ordinary charged selector, then
  restore document order and remove duplicate node identities before template
  dispatch.
- Add a golden transform whose lexical union order is `$second | $first` and
  verify the result follows source document order.
- Run the unchanged Lotus `select_select66` case and the complete conserved
  3,173-case measurement.

## Result

The golden transform produces `<out>abc</out>` from variables presented in
reverse lexical order. Unchanged Lotus `select_select66` initializes, executes,
and exactly matches its expected XML result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,124 | 2,125 | +1 |
| Initialization failures | 1,011 | 1,010 | -1 |
| Executed successfully | 2,035 | 2,036 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,904 | 1,905 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now
`1,905 / 3,173 = 60.04%`. The generic `FXXP1001` initialization frontier
falls from 22 to 19 cases. This is local compatibility evidence against a
non-redistributed archival suite, not a broad conformance claim.

## Boundaries

- Every union member must already be an admitted XSLT 1.0 key lookup, source
  location path, or source-node variable.
- The eight-member bound remains unchanged.
- This does not admit a general XPath union expression, atomic or temporary
  tree variables, new resource authority, or a public expression model.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_multiple_variable_union_restores_document_order
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
