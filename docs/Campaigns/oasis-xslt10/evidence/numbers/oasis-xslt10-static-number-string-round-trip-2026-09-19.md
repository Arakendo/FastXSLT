# OASIS XSLT 1.0 Static Number/String Round Trip

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the XSLT 1.0 compatibility path preserve distinguishable IEEE-double
values when a source-free decimal is converted through `number()` and back
through `string()`, without widening the modern expression language?

## Implemented slice

Yes. In exact XSLT 1.0 static context, the bounded source-free form
`string(number(decimal))` now:

1. validates the existing admitted decimal lexical form;
2. converts it through Rust's IEEE-754 `f64` representation;
3. produces the shortest round-tripping decimal lexical form; and
4. retains that string as a compile-time value.

Equality between two such forms folds to a typed boolean constant. Zero is
normalized to `0`. Non-finite values and results requiring exponential lexical
formatting remain outside this narrow slice. The equivalent XSLT 3.0 form
continues to report an explicit unsupported expression.

## Corpus result

The unchanged Microsoft
`XSLTFunctions_RoundTripNumber_UsingStringFn` case now preserves its two
distinct decimal values and matches its expected XML result exactly. Against
the conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,047 | 2,048 | +1 |
| Initialization failures | 1,088 | 1,087 | -1 |
| Executed successfully | 1,945 | 1,946 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,816 | 1,817 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

Focused numeric tests cover distinguishable round-trip values and two source
lexicals that collapse to the same IEEE double. A runtime test covers the
compiled boolean and value forms together while preserving explicit modern
rejection. The complete corpus measurement conserves every catalog identity
and adds no mismatch, execution failure, or panic.
