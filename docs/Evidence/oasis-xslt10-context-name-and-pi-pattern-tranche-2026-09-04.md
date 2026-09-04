# OASIS XSLT 1.0 Context Name and PI Pattern Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 485 definite unchanged XML passes; 719 initialized cases |
| Result | 514 definite unchanged XML passes; 777 initialized cases |
| Disposition | Shared XPath/XSLT semantic expansion; not a conformance claim |

## Change

FastXSLT now compiles `processing-instruction('target')` template patterns to a
typed target-bearing match rule. Its XSLT default priority is the exact-name
priority, so it outranks `processing-instruction()` without an explicit
priority. Source and temporary-tree dispatch both compare the retained target;
retained-capacity accounting includes that string.

The shared location-path parser also accepts XPath 1.0's literal
`processing-instruction('*')` form. The asterisk is a literal target, not a
wildcard, and therefore selects no possible XML processing instruction. No
special execution branch is required: it uses the same named-PI step and target
comparison.

Finally, no-argument `name()` now lowers to the existing context-node-name
operation already used by `name(.)`. The operation remains charged and retains
the existing honest boundary for namespaced nodes whose source lexical prefix
cannot be reconstructed from the current expanded-name representation.

A focused end-to-end regression proves that a named PI template outranks the
generic node test and that no-argument `name()` returns the PI target. A focused
XPath regression proves that the literal-star form produces an empty sequence.

## Measurement

The complete hash-verified local sweep moves as follows:

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 719 | 777 | +58 |
| Execution succeeded | 544 | 592 | +48 |
| XML comparison passes | 485 | 514 | +29 |
| XML comparison mismatches | 30 | 48 | +18 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 175 | 185 | +10 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **514 / 2,742 = 18.75%** of standard-operation
cases and **514 / 3,173 = 16.20%** of the complete catalog. Unchanged
`Lotus/node_node03#1` reaches a definite XML comparison pass through the named
PI selection, named pattern, `name()`, and PI-copy path.

The widened initialization frontier is deliberately not described as 58 new
passes. Eighteen cases expose later comparison mismatches and ten expose later
execution failures. One expected-error case, `Microsoft/Output__78176#1`, now
executes successfully rather than failing: its context-free global selections
expose a separate missing dynamic-context obligation. The measurement harness
now prints every expected-error unexpected-success identity so that such cases
cannot hide behind an aggregate count.

## Boundaries

This tranche does not reconstruct namespace prefixes for `name()`, add
argument-bearing `name()` generally, admit arbitrary function calls in paths,
or resolve the newly exposed mismatches and execution failures. It also does
not credit expected-error unexpected successes. Those observations remain
visible work for the compatibility profile rather than being folded into the
pass numerator.
