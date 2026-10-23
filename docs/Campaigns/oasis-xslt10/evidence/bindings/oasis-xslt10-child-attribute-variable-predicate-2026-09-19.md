# OASIS XSLT 1.0 Child Attribute/Variable Predicate

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 named template test a child attribute against the string value
of a constructed parameter without introducing a general predicate evaluator?

## Implemented slice

Yes. Exact XSLT 1.0 static context now admits the effective-boolean path
`child[@attribute=string($variable)]`, with an optional leading `./`. The plan
retains only three unqualified names. At execution it:

1. converts the variable once through the existing XSLT 1.0 atomic,
   source-node, empty-sequence, or temporary-tree string rules;
2. scans matching child elements and their attributes under the current source
   context; and
3. charges every visited node and attribute to XPath work.

Qualified names, deeper paths, other comparison operators, composed
predicates, and the equivalent modern expression remain outside the admitted
slice.

## Corpus result

The unchanged Lotus `namedtemplate07` case now selects the expected actions
for four constructed template parameters and matches its expected XML result
exactly. Against the conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,048 | 2,049 | +1 |
| Initialization failures | 1,087 | 1,086 | -1 |
| Executed successfully | 1,946 | 1,947 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,817 | 1,818 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

A focused runtime test exercises matching and nonmatching constructed
parameters and retains an explicit modern-version rejection. The unchanged
corpus case provides the recursive named-template integration proof. The full
measurement conserves every catalog identity and adds no mismatch, execution
failure, or panic.
