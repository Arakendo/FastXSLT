# OASIS XSLT 1.0 boolean node-set axis predicates -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XSLT 1.0 boolean-to-node-set comparisons in path predicates reuse the
typed location-path evaluator while retaining explicit compatibility selection,
bounded work, and modern XPath semantics?

## Decision in the experiment

The XSLT 1.0 path entry point recognizes four symmetric equality/inequality
forms whose result is exactly the effective boolean value of
`following-sibling::*`. It normalizes those forms before invoking the shared
typed path parser. The generic path model now owns a following-sibling existence
predicate and charges every examined sibling as XPath work.

The same measurement advanced two ordinary attribute-wildcard predicates.
Their first run exposed that wildcard syntax had been admitted without wildcard
matching and produced two new mismatches. The shared attribute-presence
predicate was completed before admission: `@*` now matches any attribute and
`not(@*)` matches only elements without attributes, with the existing charged
attribute scan.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,580 | 1,586 | +6 |
| Executed successfully | 1,409 | 1,415 | +6 |
| Expected-result XML matches | 1,289 | 1,295 | +6 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The following unchanged cases now match exactly:

- `Lotus/predicate_predicate03#1`
- `Lotus/predicate_predicate17#1`
- `Lotus/predicate_predicate23#1`
- `Lotus/predicate_predicate24#1`
- `Lotus/predicate_predicate25#1`
- `Microsoft/NamedTemplates__84046#1`

The strict standard-operation lower bound is now
`1,295 / 2,742 = 47.23%`; the conservative all-catalog ratio is
`1,295 / 3,173 = 40.81%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not add a general XPath 1.0 comparison evaluator. The legacy
normalization admits only the four measured following-sibling boolean forms;
other dynamic node-set comparisons remain explicit frontiers. Direct node-set
effective-boolean-value predicates and attribute wildcards use the shared path
semantics. No corpus bytes or expected results changed.

## Verification

- Focused path tests cover charged following-sibling existence and positive and
  negative attribute-wildcard presence.
- A focused runtime test covers all four legacy comparison spellings through
  `xsl:value-of` and `xsl:for-each`.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
