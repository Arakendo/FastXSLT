# OASIS XSLT 1.0 Source-Node Union Variable

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a local variable retain the union of source-node variables and can
`count($variable)` observe that normalized sequence without introducing a
second runtime value store?

## Changes

- A select-based local variable can now retain a union whose operands are
  source-node variables already visible in the invocation frame.
- Runtime binding concatenates the operand sequences, restores source document
  order, removes duplicate node identity, and stores the result through the
  existing copy-on-write source-node map.
- Under XSLT 1.0 compatibility, `count($variable)` can count that typed
  source-node sequence directly.
- Union evaluation and variable counting retain explicit XPath-operation work
  charges. Missing or non-node operands fail rather than being coerced into a
  plausible sequence.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,472 | 1,473 | +1 |
| Executed successfully | 1,310 | 1,311 | +1 |
| Expected-result XML matches | 1,196 | 1,197 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 162 | 162 | 0 |

The unchanged `Lotus/select_select72#1` case now proves that `$var1 | $var1`
retains one node and that both `count($var1)` and `count($var2)` return one.
The strict standard-operation lower bound becomes
`1,197 / 2,742 = 43.65%`; the conservative all-catalog ratio becomes
`1,197 / 3,173 = 37.72%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche admits variable-only local node unions over the current prepared
source document. It does not add arbitrary expression operands, temporary-tree
or atomic union, cross-document ordering, general sequence types, or modern
`count()` widening. Those remain separate semantic work.

## Verification

- A focused runtime test unions two source-node variables in reverse and
  repeated order, proves normalized `count()` equals two, and iterates `AB` in
  document order.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
