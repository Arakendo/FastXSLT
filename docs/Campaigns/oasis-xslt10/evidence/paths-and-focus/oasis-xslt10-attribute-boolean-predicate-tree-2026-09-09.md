# OASIS XSLT 1.0 attribute boolean predicate tree -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier and decomposition evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can attribute-presence and literal-value predicates compose with XPath `and`
and `or` precedence without flattening their semantics or creating a general
expression backend inside the path evaluator?

## Decision in the experiment

Location paths may retain one private typed boolean tree whose leaves are
attribute presence or literal equality tests and whose branches are short-
circuiting `and` and `or`. Parentheses and XPath's higher precedence for `and`
are resolved during parsing. Every visited attribute remains charged through
the existing controlled attribute scan.

The tree is used only when an attribute predicate contains `or`; simpler path
predicates retain their existing compact representation. It is shared by
modern and XSLT 1.0 paths because these admitted effective-boolean-value
semantics agree.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,588 | 1,599 | +11 |
| Executed successfully | 1,417 | 1,428 | +11 |
| Expected-result XML matches | 1,297 | 1,308 | +11 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate14#1` through
`Lotus/predicate_predicate16#1` and `Lotus/predicate_predicate49#1` through
`Lotus/predicate_predicate56#1` cases now match exactly. The strict
standard-operation lower bound is now `1,308 / 2,742 = 47.70%`; the
conservative all-catalog ratio is `1,308 / 3,173 = 41.22%`. These are local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Decomposition checkpoint

The first implementation pushed `path_experiment.rs` to 2,083 lines. Before
commit, boolean-tree representation, parsing, retained-capacity accounting,
and evaluation moved into a private predicate module, initially the 110-line
`path_attribute_predicate.rs`. The parent returned to 1,996 lines. The module
was subsequently renamed `path_boolean_predicate.rs` when the same tree gained
bounded descendant node-set/string leaves. The extracted module consumes only
path-owned parsing and charged attribute-scan helpers; it owns no XSLT compiler,
runtime frame, host policy, resource authority, or alternate evaluator.

## Boundaries

This tranche does not admit `not`, path-valued leaves, arbitrary comparisons,
functions, variables, or a public predicate representation. It does not change
result order, node identity, diagnostics, cancellation, or resource authority.
No corpus bytes or expected results changed.

## Verification

- A focused path test distinguishes all three precedence/parenthesis shapes
  represented by the newly admitted OASIS cases.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
