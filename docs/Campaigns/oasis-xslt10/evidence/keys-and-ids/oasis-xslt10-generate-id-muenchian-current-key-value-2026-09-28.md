# OASIS XSLT 1.0 `generate-id()` Muenchian Grouping and `current()` Key Value -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Lotus `idkey_idkey01` case uses the classic XSLT 1.0 grouping
predicate `generate-id() = generate-id(key('places', @state)[1])`. Once that
predicate selected each first keyed node, its nested iteration looked up the
complete group with `key('places', current()/@state)`.

FastXSLT already owned the equivalent count/union Muenchian plan, stable source
node identity, an invocation-owned key index, and typed context paths. The case
therefore pressured alternate syntax over existing semantics rather than a new
grouping or caching model.

## Change

The compiler now recognizes either operand order of the bounded
`generate-id()` equality when one side is the zero-argument current-node form
and the other is a complete typed `key(...)[1]` lookup. It lowers the
predicate to the same private first-by-key selection used by the count/union
form.

A key lookup value beginning with `current()/` now lowers the remaining
location path as the existing typed context path. Runtime evaluation uses the
lookup call's explicit source-node focus. No ambient current-node state,
general identity comparison, general `current()` expression, or additional
key evaluator was introduced.

## Corpus result

Unchanged `Lotus/idkey_idkey01#1` initializes, executes, and compares exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,424 | 2,425 | +1 |
| Executed successfully | 2,374 | 2,375 | +1 |
| Initialization failures | 746 | 745 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact XML expected-result matches | 2,213 | 2,214 | +1 |
| Visible comparator gaps | 53 | 53 | 0 |

The conservative exact-match ratio is `2,214 / 3,173 = 69.78%`.
Expected-error credit remains 423 / 431.

## Verification

Focused compiler and runtime tests cover the alternate grouping syntax and
`current()/@state` lookup value independently. The unchanged corpus case then
proves their composition against the archival expected result.

```powershell
cargo test -p fastxslt --all-features xslt10_generate_id_muenchian
cargo test -p fastxslt --all-features xslt10_key_lookup_current_context_path
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/idkey_idkey01#1'
./scripts/verify.ps1
```

## Normative references

- [XSLT 1.0 section 12.2, Keys](https://www.w3.org/TR/1999/REC-xslt-19991116#keys)
- [XSLT 1.0 section 12.4, Miscellaneous Additional Functions](https://www.w3.org/TR/1999/REC-xslt-19991116#misc-func)
