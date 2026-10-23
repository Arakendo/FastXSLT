# OASIS XSLT 1.0 Exact Integral Arithmetic Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 738 definite unchanged XML passes; 1,022 initialized cases |
| Result | 747 definite unchanged XML passes; 1,031 initialized cases |
| Disposition | Shared checked constant folding; not a conformance claim |

## Change

The value-expression compiler now exposes the existing checked exact-rational
parser for source-free binary arithmetic when the final result is exactly
integral. The admitted operators are the parser's ordinary `+`, `-`, `*`,
`div`, and `mod` operations, including unary signs and parentheses already
handled by that parser. A successful expression is compiled once to its
canonical integer lexical result; transformation performs no arithmetic or
version dispatch.

Fractional results, division by zero, overflow, NaN/infinity, path operands,
and other numeric conversion or formatting semantics remain outside this
constant-folding slice. The new compiler probe also requires a binary operator,
so a bare numeric literal is not reclassified through this path.

## Unchanged cases

Nine standard-operation cases move directly to definite XML comparison passes:

- `Lotus/math_math06#1` (`2*3`);
- `Lotus/math_math07#1` (`3+6`);
- `Lotus/math_math09#1` (`6 div 2`);
- `Lotus/math_math10#1` (`5 mod 2`);
- `Lotus/math_math60#1` (`1-2`);
- `Lotus/math_math62#1` (`7+-3`);
- `Lotus/math_math64#1` (`7 - -3`);
- `Lotus/math_math66#1` (`-7 --3`); and
- `Lotus/math_math70#1` (`6 div -2`).

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,022 | 1,031 | +9 |
| Execution succeeded | 821 | 830 | +9 |
| XML comparison passes | 738 | 747 | +9 |
| XML comparison mismatches | 50 | 50 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 201 | 201 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **747 / 2,742 = 27.24%** of standard-operation
cases and **747 / 3,173 = 23.54%** of the complete catalog.

## Verification

A focused numeric test covers multiplication, addition, exact division,
modulo, unary-sign composition, rejection of a fractional quotient, rejection
of division by zero, and refusal to claim a bare literal. A runtime test drives
representative forms through normal stylesheet compilation and transformation.
The complete local OASIS sweep confirms all nine affected identities pass with
no new mismatch, comparator gap, runtime failure, unexpected success, or panic.
