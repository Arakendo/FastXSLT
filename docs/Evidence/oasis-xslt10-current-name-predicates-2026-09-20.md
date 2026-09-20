# OASIS XSLT 1.0 Current-Name Predicates

Date: 2026-09-20  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the XSLT 1.0 compatibility path retain the outer `current()` node while a
predicate establishes a different focus, without introducing ambient runtime
state or widening the general XPath grammar?

## Implemented slice

Yes. Three exact, typed compatibility plans now cover the shared same-name
operations used by the Microsoft `84424` and `84426` cases:

- the effective boolean value of
  `descendant::*[name()=name(current())] |
  following::*[name()=name(current())]`;
- `count(/descendant::*[name()=name(current())])`; and
- `//*[name()=name(current())]/*` as an `xsl:for-each` or
  `xsl:apply-templates` selection.

Each plan receives the expression-entry source node explicitly. Predicate
candidate traversal never replaces that outer node for `current()`. Lexical
QName comparison preserves both the expanded name and retained source prefix.
Descendant, following, rooted-document, and selected-child visits are charged
before use. The selection remains in source document order and does not retain
cross-invocation state.

This is not a general function-call or predicate-composition grammar. It does
not admit `current()` in modern static context, arbitrary operands to `name()`,
namespace-insensitive comparison, or other composed `current()` expressions.

## Corpus result

The unchanged Microsoft `Miscellaneous__84424` and
`Miscellaneous__84426` cases now execute their original stylesheets and match
their expected results exactly. Against the conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,053 | 2,055 | +2 |
| Initialization failures | 1,082 | 1,080 | -2 |
| Executed successfully | 1,951 | 1,953 | +2 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,822 | 1,824 | +2 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed. The
remaining `FXXP1002` initialization frontier contains five cases: four
forward-compatible version-policy cases and one broad nested-predicate/node-set
comparison case.

## Verification

A focused regression uses repeated nested and following element names. It
proves that the boolean plan rejects earlier occurrences, the rooted count
retains the original expression node, and the selection gathers children of
all same-named elements while its body receives ordinary per-item focus. The
two unchanged corpus cases additionally exercise template dispatch, HTML result
construction, rooted descendant counts, and mode-specific application.

