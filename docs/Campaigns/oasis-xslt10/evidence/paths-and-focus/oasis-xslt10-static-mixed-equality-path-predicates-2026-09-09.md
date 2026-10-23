# OASIS XSLT 1.0 static mixed-equality path predicates -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can statically decidable XSLT 1.0 equality predicates filter an ordinary typed
location path without introducing dynamic node-set comparison or a second XPath
evaluator?

## Decision in the experiment

In XSLT 1.0 compatibility mode, a single predicate whose equality is completely
determined by literal operands is folded during compilation using the existing
XPath 1.0 mixed-type equality rules. A true predicate retains the ordinary
typed location path; a false predicate selects the typed empty path. Execution
therefore continues through the existing first-node path-value operation.

The admitted slice covers boolean-to-number, boolean-to-string, and
number-to-string literal equality. It does not admit dynamic node-set equality,
relational predicates, multiple predicates, or runtime compatibility branches.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,577 | 1,580 | +3 |
| Executed successfully | 1,406 | 1,409 | +3 |
| Expected-result XML matches | 1,286 | 1,289 | +3 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate01#1`,
`Lotus/predicate_predicate02#1`, and `Lotus/predicate_predicate04#1` cases now
match exactly. The strict standard-operation lower bound is now
`1,289 / 2,742 = 47.01%`; the conservative all-catalog ratio is
`1,289 / 3,173 = 40.62%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not select general predicate expression evaluation or change
modern XPath comparison semantics. Source-dependent node-set comparisons stay
on their existing typed paths or remain explicitly unsupported. No corpus bytes
or expected results changed.

## Verification

- A focused runtime test proves true and false boolean/literal predicates plus
  numeric/string equality over one typed child path.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
