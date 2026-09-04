# OASIS XSLT 1.0 Integral-Function Path Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 638 definite unchanged XML passes; 904 initialized cases |
| Result | 647 definite unchanged XML passes; 913 initialized cases |
| Disposition | Shared typed path and numeric semantics; not a conformance claim |

## Change

The value compiler now composes `floor()`, `ceiling()`, and `round()` with the
existing typed location-path representation. Runtime evaluation uses the
charged path evaluator, requires a zero-or-one node result, obtains the
complete charged XDM string value, and applies the same checked finite-decimal
numeric operation used by constant folding.

The compiled operation owns its path, function kind, and diagnostic source
location. Its retained path and location capacity participate in the existing
prepared-engine accounting model. The numeric operation charges its own XPath
work after navigation and string-value work have been charged by their owners.

## Unchanged cases

Nine Lotus cases move directly from initialization failure to XML comparison
pass:

- `math23`, `math25`, and `math27` for `floor()`;
- `math32`, `math34`, and `math36` for `ceiling()`;
- `math40`, `math42`, and `math44` for `round()`.

These cases select exactly one element containing a finite decimal. This
tranche does not admit general sequences, legacy first-node conversion, NaN,
infinity, exponent notation, string operands, or general numeric expressions.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 904 | 913 | +9 |
| Execution succeeded | 716 | 725 | +9 |
| XML comparison passes | 638 | 647 | +9 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 188 | 188 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **647 / 2,742 = 23.60%** of
standard-operation cases and **647 / 3,173 = 20.39%** of the complete catalog.

## Verification

A first-party production-path test covers positive and negative path values,
all three functions, and an empty path. A separate boundary test proves that a
two-node path reports `XPTY0004` instead of silently selecting its first node.
The complete local OASIS sweep confirms that all nine newly initialized cases
pass without creating a later failure or comparison mismatch.
