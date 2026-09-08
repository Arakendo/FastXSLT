# OASIS XSLT 1.0 Variable/Path Apply Union

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:apply-templates` combine an invocation-local source-node variable with
ordinary location paths while preserving XPath union identity, document order,
focus, and the existing resource and work-control boundaries?

## Changes

- Apply-selection compilation now retains one distinct source-node variable
  alongside one or more typed location-path alternatives.
- Repeated references to that same variable are admitted; a second distinct
  variable remains explicitly unsupported until general sequence union has a
  broader typed owner.
- Runtime selection combines the variable's source nodes with the controlled
  path results, restores document order, and removes duplicate node identity
  before establishing template focus.
- Atomic and temporary-tree variables do not masquerade as source node sets.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,471 | 1,472 | +1 |
| Executed successfully | 1,309 | 1,310 | +1 |
| Expected-result XML matches | 1,195 | 1,196 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 162 | 162 | 0 |

The unchanged `Lotus/select_select65#1` case now combines `$var1` with
`child::child2`, applies each selected node once, and preserves source document
order. The strict standard-operation lower bound becomes
`1,196 / 2,742 = 43.62%`; the conservative all-catalog ratio becomes
`1,196 / 3,173 = 37.69%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not add general sequence union, union-valued variable
bindings, temporary-tree union, cross-document ordering, completion-order
template application, or a new path evaluator. It is a private typed
apply-selection form over nodes from the current prepared source document.

## Verification

- A focused runtime test combines a repeated source-node variable with an
  earlier path result and proves document-order and identity normalization.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
