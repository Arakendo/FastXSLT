# OASIS XSLT 1.0 Same-Name Local/Global Alias

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can a direct local-variable initializer resolve a same-named global binding
before the local shadow is installed, without reducing non-atomic values to
strings or adding a second XSLT 1.0 value model?

## Method

- Generalize the private direct-variable alias plan from atomic-only naming to
  value-kind preservation.
- Resolve an alias from earlier lexical locals first, then from globals only
  while the name has not already been shadowed locally.
- Preserve atomic values, atomic sequences, source-node sequences, empty
  sequences, and temporary trees through the existing runtime value stores.
- Add a focused regression in which local `$value` initializes from a
  same-named global temporary tree composed from text and another global.
- Run the hash-verified 3,173-case OASIS CD04 measurement and trace unchanged
  Microsoft case `BVTs_bvt036`.

## Result

The focused regression produces `<out>global tree</out>`. The unchanged OASIS
case produces `global foo: global bar`, exactly matching its expected result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,096 | 2,096 | 0 |
| Initialization failures | 1,039 | 1,039 | 0 |
| Executed successfully | 2,008 | 2,009 | +1 |
| Execution failures | 88 | 87 | -1 |
| Exact XML-semantic matches | 1,877 | 1,878 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The measured exact compatibility lower bound is now
`1,878 / 3,173 = 59.19%`. The `FXRT0002` execution frontier falls from 10 to 9
cases. This is local compatibility evidence against the hash-verified,
non-redistributed OASIS CD04 archive; it is not a broad conformance claim.

## Boundaries

- The initializer resolves before its target local binding is installed.
- A prior lexical local shadow prevents fallback to a same-named global.
- Temporary-tree aliases preserve their semantic identity within the current
  invocation; no cross-invocation, worker, snapshot, or generation sharing is
  introduced.
- General variable expressions remain outside this direct-alias slice.

## Reproduction

```powershell
cargo test -p fastxslt --all-features local_variable_initializer_can_reference_a_same_named_global
./scripts/measure-oasis-xslt10.ps1 -TraceCase BVTs_bvt036
./scripts/verify.ps1
```
