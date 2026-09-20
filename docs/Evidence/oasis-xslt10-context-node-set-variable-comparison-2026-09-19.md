# OASIS XSLT 1.0 Context/Node-Set Variable Comparison

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the XSLT 1.0 compatibility path compare the current source node with a
source-node variable using node-set value semantics rather than identity or a
first-node approximation?

## Implemented slice

Yes. The exact XSLT 1.0 boolean form `. = $variable` now requires the variable
to hold a source-node sequence and applies XPath 1.0 node-set equality:

1. the current source node's controlled string value is computed once;
2. every variable node is converted through the controlled source string-value
   path until a match is found; and
3. the comparison returns true if any pair matches.

The variable name and source location are retained in compiled state. Missing
or non-source-node variables remain typed runtime errors. Atomic values,
temporary trees, other operators, reversed operands, and modern static context
remain outside this bounded form.

## Corpus result

The unchanged Lotus `variable51` case now compares document-node parameters
inside a variable-selected `xsl:for-each` and matches its expected result
exactly. Against the conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,050 | 2,051 | +1 |
| Initialization failures | 1,085 | 1,084 | -1 |
| Executed successfully | 1,948 | 1,949 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,819 | 1,820 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

A focused runtime regression distinguishes node-set string-value equality from
node identity by matching two different elements with equal text while
rejecting a different string value. It also preserves explicit modern
rejection. The unchanged corpus case supplies document-node and parameter-flow
coverage. Full corpus accounting remains conserved without a new mismatch,
execution failure, or panic.
