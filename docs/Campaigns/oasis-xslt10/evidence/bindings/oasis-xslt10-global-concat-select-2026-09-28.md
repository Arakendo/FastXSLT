# OASIS XSLT 1.0 Global `concat()` Select -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Microsoft `BVTs_bvt037` stylesheet defines several XSLT 1.0
global variables whose `select` expressions concatenate string literals and
other global variables. Some dependencies are declared later in stylesheet
order. FastXSLT already had a bounded typed `concat()` plan for local
expressions, but top-level binding compilation fell through to the location-
path parser and reported valid XPath as `XPST0003`.

## Change

XSLT 1.0 global bindings now reuse the existing typed concat plan when every
argument is a string literal or an unqualified variable reference. The global
dependency sorter observes every variable part before invocation-local
materialization. Execution then evaluates the plan in dependency order,
charges XPath work, resolves values through the ordinary global value domain,
and binds the result as `xs:string`.

The admitted global form remains intentionally narrower than local concat:
path, sum-path, and positional-variable arguments remain outside this global
slice rather than acquiring an alternate evaluator accidentally. Modern
stylesheet compilation is unchanged.

## Corpus result

Unchanged `Microsoft/BVTs_bvt037#1` now initializes, executes, and compares
exactly. The complete conserved sweep reports:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,421 | 2,422 | +1 |
| Executed successfully | 2,371 | 2,372 | +1 |
| Initialization failures | 749 | 748 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,211 | 2,212 | +1 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,212 / 3,173 = 69.71%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52.

Three additional stylesheets with concat-based cyclic global dependencies now
reach the existing `XTDE0640` cycle classifier instead of being mislabeled as
XPath syntax failures. This classification shift does not change expected-
error credit.

## Boundaries

- Global dependency ordering remains a compile-time property; no lazy global
  cache or cross-invocation state was introduced.
- The plan retains only stylesheet-derived literals and variable names.
- Runtime values remain invocation-owned and use the existing work-control and
  structured-failure paths.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_global_concat
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt037#1'
./scripts/verify.ps1
```

## Normative reference

- [XPath 1.0 section 4.2, String Functions](https://www.w3.org/TR/1999/REC-xpath-19991116/#section-String-Functions)
