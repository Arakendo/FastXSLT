# OASIS XSLT 1.0 local path-arithmetic variable -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a local variable bind source-dependent numeric arithmetic without creating
a second arithmetic implementation or weakening typed runtime frames?

## Implemented slice

An XSLT 1.0 local variable whose `select` is admitted by the existing bounded
binary-numeric compiler now retains that same private expression plan. Runtime
binding evaluates it with the existing controlled path, XPath work charging,
first-node XSLT 1.0 conversion, exact-rational arithmetic, and typed failure
mapping, then stores the result as the existing `xs:double` atomic value.

Static atomic bindings and aliases keep their established paths. The new branch
runs only after those cheaper classifications and before generic node-path and
cast compilation. Retention accounting owns the boxed expression explicitly;
semantic inspection continues to report only the stable local-variable feature.

## Corpus result

`Lotus/variable_variable43` leaves `FXXP1008` and becomes an exact match. Its
`width * depth` expression reads the first selected node for each operand under
XPath 1.0 compatibility semantics.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,700 | 1,701 | +1 |
| Initialization failures | 1,435 | 1,434 | -1 |
| Executed successfully | 1,517 | 1,518 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,384 | 1,385 | +1 |
| XML comparison mismatches | 106 | 106 | 0 |
| `FXXP1008` initialization frontier | 7 | 6 | -1 |

The exact compatibility lower bound is now
`1,385 / 2,742 = 50.51%` for standard-operation cases and
`1,385 / 3,173 = 43.65%` for the complete catalog.

## Boundaries

This does not create a general local-variable expression compiler. It admits
only expressions already recognized by the bounded XSLT 1.0 binary-numeric
plan, and it does not add `document()`, `current()`, variable-rooted paths, or
arbitrary function calls. The two extreme fractional-literal cases remain
outside the exact-rational parser's bounded representation and are not hidden
by floating-point approximation.

## Verification

- A focused runtime test binds `width * depth` from source children and observes
  `42` through ordinary variable lookup.
- The complete 3,173-case measurement moves exactly one case from
  initialization failure to exact result without a new mismatch or execution
  failure.
- Formatting, strict Clippy, the complete workspace suite, documentation, link,
  unsafe-surface, and corpus-inventory checks pass through `scripts/verify.ps1`.
