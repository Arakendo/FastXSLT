# OASIS XSLT 1.0 Variable Position Path

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 numeric variable supply the position predicate for a simple
child selection without approximating the different focus semantics of a
multi-step path?

## Change

The value-expression compiler now recognizes these bounded XSLT 1.0 forms for
one child step:

```xpath
a[$third]
a[position() = $first]
a[$third = position()]
```

Execution resolves the variable through the existing XSLT 1.0 string/number
conversion path, charges the selection work, and selects only a finite,
positive integral position. `NaN`, infinity, fractions, zero, and negative
values select no node.

The compiler deliberately rejects paths containing `/`. Applying a predicate
to a fully materialized multi-step result is not equivalent to XPath's
per-step focus. In particular, `.//description[$pos]` remains explicitly
unsupported until the general path plan can preserve that boundary.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,445 | 1,448 | +3 |
| Executed successfully | 1,284 | 1,287 | +3 |
| Expected-result XML matches | 1,170 | 1,173 | +3 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are:

- `Lotus/position_position18#1`
- `Lotus/position_position96#1`
- `Lotus/variable_variable21#1`

`Lotus/position_position97#1` remains an explicit `FXXP1001` initialization
failure because its descendant path requires per-step predicate focus.

The strict standard-operation lower bound becomes
`1,173 / 2,742 = 42.78%`; the conservative all-catalog ratio becomes
`1,173 / 3,173 = 36.97%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused runtime test covers direct-variable, both position-equality
  spellings, and a fractional nonmatch.
- A compiler regression proves that a descendant variable-position expression
  stays unsupported rather than producing an approximation.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
