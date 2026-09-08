# OASIS XSLT 1.0 Static String Local and Parameter-Default Tranche

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can static string-valued local bindings and template-parameter defaults reuse
the typed atomic runtime while preserving lexical scope and XSLT 1.0
string-compatible comparison behavior?

## Changes

- A local `xsl:variable select="'literal'"` compiles to the existing typed
  atomic-variable instruction. The value remains invocation-local and obeys
  the existing lexical frame and shadowing rules.
- A two-literal static string function returning a string can supply a
  template parameter default through the existing text-default representation.
- XSLT 1.0 `xsl:value-of` can compare two string-compatible variables with
  `=` or `!=`. Evaluation uses the existing variable string-value operation
  and produces the standard boolean lexical result.

The comparison slice does not claim general node-set-to-node-set equality;
that requires independently existential pair comparison rather than scalar
string conversion and remains outside this tranche.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,453 | 1,458 | +5 |
| Executed successfully | 1,292 | 1,297 | +5 |
| Expected-result XML matches | 1,178 | 1,183 | +5 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

Representative unchanged exact cases directly traced for this tranche are:

- `Lotus/variable_variable19#1`
- `Lotus/variable_variable20#1`
- `Lotus/variable_variable36#1`
- `Microsoft/Variables_VariableScopeWithinLRE#1`

An earlier draft incorrectly included `Lotus/variable_variable44#1` in this
list. A later targeted trace showed that its `$b := $a` local binding remained
unsupported at this checkpoint; the subsequent sum-composition and atomic-alias
tranche admits and credits that case.

The strict standard-operation lower bound becomes
`1,183 / 2,742 = 43.14%`; the conservative all-catalog ratio becomes
`1,183 / 3,173 = 37.28%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused runtime tests cover lexical shadowing of a string-literal local and
  the static-function parameter default plus variable comparison.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
