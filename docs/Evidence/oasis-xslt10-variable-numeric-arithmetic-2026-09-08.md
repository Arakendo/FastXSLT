# OASIS XSLT 1.0 Variable Numeric Arithmetic

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing checked numeric-expression tree compose XSLT 1.0 variables
with source paths and literals without introducing a second arithmetic
evaluator or changing modern stylesheet behavior?

## Changes

- The private binary numeric tree can retain an unprefixed variable operand
  when XSLT 1.0 static context selects compatibility conversion.
- Runtime variable resolution reuses the invocation-local atomic, source-node,
  temporary-tree, empty, and global-fallback owner. Resolution failures retain
  their existing structured diagnostic rather than being collapsed into a
  numeric error.
- Each resolved variable and arithmetic operation remains work charged. The
  checked exact-rational representation, path-selection policy, failure order,
  and final formatting are unchanged.
- Modern static context does not admit the compatibility variable operand;
  this tranche therefore does not silently apply XSLT 1.0 first-node
  conversion to modern stylesheets.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,462 | 1,465 | +3 |
| Executed successfully | 1,300 | 1,303 | +3 |
| Expected-result XML matches | 1,186 | 1,189 | +3 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 162 | 162 | 0 |

The unchanged exact cases are:

- `Lotus/math_math08#1`
- `Lotus/math_math97#1`
- `Lotus/math_math100#1`

The strict standard-operation lower bound becomes
`1,189 / 2,742 = 43.36%`; the conservative all-catalog ratio becomes
`1,189 / 3,173 = 37.47%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit general variable expressions, namespaced variable
references, variable-rooted paths, calls such as `number($value)`, focus
functions as arithmetic leaves, non-finite arithmetic, or a general XPath
parser. The adjacent repeated-subtraction, multiplication, and modulo cases
retain their independently visible grammar/function boundaries.

## Verification

- A focused runtime test composes a global numeric variable with two source
  paths and literals through the checked tree.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
