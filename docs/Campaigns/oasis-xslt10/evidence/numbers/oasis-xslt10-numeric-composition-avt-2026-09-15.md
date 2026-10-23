# OASIS XSLT 1.0 Numeric-Composition AVT -- 2026-09-15

Date: 2026-09-15  
Status: Verified shared semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a multi-part literal-result attribute value template reuse the existing
XSLT 1.0 exact-rational numeric evaluator rather than introducing a separate
attribute-only arithmetic implementation?

## Implemented slice

The private typed AVT part model now includes an already-compiled binary
numeric expression. Compilation delegates to the shared XSLT 1.0 binary
numeric compiler, including its first-node-in-document-order conversion rule.
Runtime materialization delegates to the shared controlled evaluator with the
same invocation variables and source focus used by ordinary value expressions.

The AVT scanner remains bounded to 32 total parts. General expression
evaluation, unsupported numeric lexicals, non-terminating exact-rational
results, and zero divisors retain the existing explicit diagnostics. Every
path visit and arithmetic operation retains the established XPath work charge.

## Corpus result

The unchanged Microsoft `AVTs__77576` case now initializes and executes. Its
two expressions, `(@size + 1)` and `@size div 2`, produce the expected decimal
attribute values for all five source elements. The case remains a visible,
uncredited comparison mismatch solely because its expected fragment contains
line breaks between top-level result elements while FastXSLT emits the same
elements contiguously. The XML comparator was not weakened.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,770 | 1,771 | +1 |
| Initialization failures | 1,365 | 1,364 | -1 |
| Executed successfully | 1,586 | 1,587 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,444 | 1,444 | 0 |
| XML comparison mismatches | 113 | 114 | +1 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound remains
`1,444 / 2,742 = 52.66%` for standard-operation cases and
`1,444 / 3,173 = 45.51%` for the complete catalog.

## Verification

A focused production runtime test compiles and executes two numeric AVT parts,
including a fractional exact result. The unchanged corpus case verifies the
same route against upstream source and expected output. The complete sweep
conserves every catalog case and introduces no execution failure or panic.
