# OASIS XSLT 1.0 Forward-Compatible Expression Deferral -- 2026-09-27

Date: 2026-09-27  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Six unchanged Microsoft `ForwardComp` cases place an invalid XPath expression,
invalid function arity, or a numeric literal where a future introspection
function requires a string in a template under version `1.5`. XSLT 1.0 section
2.5 requires errors in expressions to be reported only if the expression is
actually evaluated while forwards-compatible processing is active.

The shared compiler previously reported these errors while compiling the whole
stylesheet. That incorrectly rejected dormant templates which were never
instantiated and attributed invoked failures to initialization rather than
execution.

## Change

When an instruction in the bounded XSLT 1.x forward-compatible interval has a
confirmed invalid XPath syntax/arity failure (`XPST0003`) or the narrowly
recognized numeric-literal introspection type failure (`XPTY0004`), the compiler
now retains a private deferred-failure instruction carrying the original code,
detail, and source location. Execution reports that failure only if control
reaches the instruction.

This is deliberately narrower than deferring an unsupported expression. A
valid expression outside the currently admitted engine surface remains
unsupported during compilation; it is not relabeled as a standards error or
silently accepted.

A focused regression covers both branches: an invalid expression in a dormant
template does not fail compilation, while the same expression in an invoked
template reports the original structured failure during execution.

## Corpus result

The unchanged cases `Microsoft/ForwardComp__91849#1` and
`Microsoft/ForwardComp__91853#1`, and `Microsoft/ForwardComp__91855#1` now
compile, execute, and compare exactly.
The expected-error cases `Microsoft/ForwardComp__91850#1` and
`Microsoft/ForwardComp__91854#1` now initialize and report `XPST0003` only when
their instructions execute; `Microsoft/ForwardComp__91856#1` similarly reports
`XPTY0004` during execution.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,410 | 2,416 | +6 |
| Executed successfully | 2,361 | 2,364 | +3 |
| Initialization failures | 760 | 754 | -6 |
| Execution failures | 49 | 52 | +3 |
| Exact expected-result matches | 2,201 | 2,204 | +3 |
| Expected errors during initialization | 388 | 385 | -3 |
| Expected errors during execution | 35 | 38 | +3 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,204 / 3,173 = 69.46%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52.

## Boundaries

- Deferral is selected statically only for the bounded XSLT 1.x
  forward-compatible interval.
- Only confirmed invalid-expression failures are deferred by this tranche.
- Valid but unsupported XPath remains visibly unsupported.
- Supported XSLT `1.0`, `2.0`, and `3.0` declarations retain ordinary eager
  static checking.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

- The focused dormant/invoked compiler and runtime regression passes.
- All three unchanged normal-result cases compare exactly.
- All three unchanged expected-error cases now fail at execution with the retained
  structured diagnostic.
- The complete 3,173-case catalog sweep produced the counters above.

## Normative reference

- [XSLT 1.0 section 2.5, Forwards-Compatible Processing](https://www.w3.org/TR/xslt-10/#forwards)
