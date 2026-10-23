# OASIS XSLT 1.0 Constant Integral Functions Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 614 definite unchanged XML passes; 880 initialized cases |
| Result | 638 definite unchanged XML passes; 904 initialized cases |
| Disposition | Shared checked numeric semantics; not a conformance claim |

## Change

The existing checked exact-rational constant evaluator now supports unary signs
and the XPath `floor()`, `ceiling()`, and `round()` operations. Direct integral
results are folded into the existing literal value path during stylesheet
compilation, while equality expressions use the same exact comparison
machinery. Negative half values preserve XPath's rounding direction toward
positive infinity.

This is a compile-time specialization over context-independent numeric
expressions. It adds no runtime evaluator branch and does not admit node-set,
string, context-item, NaN, infinity, or XSLT 1.0 compatibility coercions.

## Unchanged cases

The following 24 Lotus cases move directly from initialization failure to XML
comparison pass:

- `math02`, `math24`, `math26`, `math28`, `math29`, `math30`, and `math31`;
- `math03`, `math33`, `math35`, `math37`, `math38`, and `math39`;
- `math04`, `math41`, `math43`, `math45` through `math52` except `math44`.

The first group exercises `floor`, the second `ceiling`, and the third `round`.
Dynamic path operands such as those in `math23`, `math25`, `math27`, `math32`,
`math34`, `math36`, `math40`, `math42`, and `math44` remain explicitly outside
this constant tranche.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 880 | 904 | +24 |
| Execution succeeded | 692 | 716 | +24 |
| XML comparison passes | 614 | 638 | +24 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 188 | 188 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **638 / 2,742 = 23.27%** of
standard-operation cases and **638 / 3,173 = 20.11%** of the complete catalog.

## Verification

A first-party runtime case executes positive and negative direct results plus
constant equalities through normal stylesheet compilation, transformation, and
serialization. Focused exact-rational tests cover rounding direction, unary
signs, integral result folding, and unsupported dynamic operands. The complete
local OASIS sweep confirms that every newly initialized case passes without
creating a later failure or mismatch.
