# OASIS XSLT 1.0 local-name path predicate -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the shared typed location-path evaluator support a literal
`local-name()` equality predicate without adding an `xsl:copy-of`-specific
runtime or weakening namespace semantics?

## Decision in the experiment

The path compiler now retains exact predicates of the forms
`local-name()='name'` and `local-name(.)='name'` as a typed context-local-name
axis predicate. Evaluation compares the candidate node's retained expanded
name, so namespaced and unnamespaced elements with the required local part are
selected together.

The ordinary path evaluator still owns candidate traversal, document-order
normalization, duplicate elimination, work charging, cancellation, and result
copying. The predicate adds one XPath-operation charge for each evaluated
candidate. No lexical namespace prefix is treated as the node's identity.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,574 | 1,575 | +1 |
| Executed successfully | 1,403 | 1,404 | +1 |
| Expected-result XML matches | 1,283 | 1,284 | +1 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/copy_copy46#1` case now matches exactly, including
namespace nodes introduced by copying each selected subtree. The strict
standard-operation lower bound is now `1,284 / 2,742 = 46.83%`; the
conservative all-catalog ratio is `1,284 / 3,173 = 40.47%`. These are local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche admits only equality with a static string literal and a current
node supplied by location-path predicate focus. Dynamic operands, other
expanded-name functions, namespace-sensitive name equality, and general
predicate expressions remain outside this slice. No corpus bytes or expected
results changed.

## Verification

- A focused XPath test selects both namespaced and unnamespaced `bar` children
  while rejecting an `other` sibling.
- A focused runtime test copies the selected subtrees in document order and
  preserves required namespace nodes.
- The complete 3,173-case local measurement produced the counters above.
