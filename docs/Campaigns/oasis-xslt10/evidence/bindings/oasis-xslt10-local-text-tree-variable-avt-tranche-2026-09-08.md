# OASIS XSLT 1.0 Local Text-Tree Variable and AVT Tranche

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT retain a text-constructed local XSLT 1.0 variable as a temporary
tree, use its string value in a bounded variable/path AVT, and preserve the
different predicate semantics of `$fragment` and `position() = $fragment`?

## Changes

- A text-only, untyped XSLT 1.0 local variable compiles to an explicit temporary
  text-tree binding rather than an atomic string.
- Runtime materializes that tree under the existing invocation-owned temporary
  identity, XDM budget, copy-on-write frame, and string-value accounting.
- Literal attributes admit exactly `{$variable}separator{path}` with one local
  variable, static separator, and typed source path.
- Direct `path[$fragment]` uses the temporary tree's XSLT 1.0 boolean value;
  explicit `path[position() = $fragment]` converts its string value to a number.
  The compiled plan retains which predicate form was written.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,479 | 1,490 | +11 |
| Executed successfully | 1,316 | 1,324 | +8 |
| Expected-result XML matches | 1,209 | 1,217 | +8 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 166 | +3 |

The exact additions are the Lotus `attribvaltemplate02`, `variable07`, and
`variable08` cases; Microsoft `Variables__78132`, `Variables__78137`,
`Variables__78160`, and `Variables__78163`; and
`Variables_VarScopeInCallTemplate`. Newly reachable unsupported or invalid
execution remains visible rather than receiving pass credit.

The strict expected-result lower bound is now 1,217 of 2,742
standard-operation cases (44.38%) and 1,217 of all 3,173 catalog cases
(38.35%). This is local compatibility evidence, not a conformance claim.

## Boundaries

This tranche does not admit general result-tree-fragment extensions, arbitrary
sequence constructors, multiple dynamic AVT expressions, global temporary-tree
lookup through this AVT shape, or broad variable-dependent path predicates.

## Verification

- A focused compiler test proves the variable/path AVT plan form.
- Focused runtime tests prove text-tree materialization, distinguish direct-
  fragment predicate truth from explicit numeric position comparison, and
  verify the exact AVT result.
- The full local measurement has no initialization panic and no new mismatch.
- No upstream corpus byte was edited.
