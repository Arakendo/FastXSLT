# OASIS XSLT 1.0 text-child predicate -- 2026-09-15

Date: 2026-09-15  
Status: Verified shared semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the typed location-path evaluator apply the XPath 1.0 node-set effective
boolean value for `[text()]` without adding an AVT-specific shortcut?

## Implemented slice

The private axis-predicate representation now distinguishes a child text-node
test from a named child-element test. Evaluation scans the candidate node's
children under the existing XPath node-visit budget and succeeds when at least
one text child exists. The explicit `child::text()` spelling uses the same
representation and execution path.

The test is based on node existence, not string content: an empty text node
still makes the predicate true. Other node-kind predicates and general
predicate expressions remain outside this slice.

## Corpus result

The unchanged Microsoft `AVTs__77570` case now initializes, executes, and
matches its expected XML. Its literal-result attribute combines four path
expressions, including `.//first-name[text()]`, under the bounded multi-path
AVT representation introduced immediately before this tranche.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,764 | 1,765 | +1 |
| Initialization failures | 1,371 | 1,370 | -1 |
| Executed successfully | 1,580 | 1,581 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,442 | 1,443 | +1 |
| XML comparison mismatches | 109 | 109 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,443 / 2,742 = 52.63%` for standard-operation cases and
`1,443 / 3,173 = 45.48%` for the complete catalog.

## Verification

A focused evaluator test covers `[text()]` and `[child::text()]`, including an
empty text node and a child-element-only negative case. The complete local
corpus sweep establishes the exact unchanged case result and conserved
denominators.
