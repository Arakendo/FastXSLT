# OASIS XSLT 1.0 descendant node-set predicate comparison -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the typed path-predicate tree admit XSLT 1.0 existential comparison of a
`descendant::*` node-set with a string literal, including `not`, `and`, `or`,
operand symmetry, and the distinct meaning of `!=`, without admitting a general
predicate evaluator?

## Decision in the experiment

The private path-predicate tree now has one descendant-element/string comparison
leaf and one `not` branch. Both `=` and `!=` use XSLT 1.0 node-set comparison:
the result is true when at least one descendant element's string value satisfies
the requested comparison. Consequently, `node-set != literal` is not lowered to
the negation of `node-set = literal`.

Descendant traversal is charged through the existing XPath node-visit domain,
short-circuiting remains explicit, and the feature is selected only for the
bounded `descendant::*`/literal form inside a final location-path predicate.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,599 | 1,605 | +6 |
| Executed successfully | 1,428 | 1,434 | +6 |
| Expected-result XML matches | 1,308 | 1,314 | +6 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate06#1`, `12#1`, `13#1`, and `20#1`
through `22#1` cases now match exactly. The strict standard-operation lower
bound is now `1,314 / 2,742 = 47.92%`; the conservative all-catalog ratio is
`1,314 / 3,173 = 41.41%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Representation checkpoint

The earlier attribute-only module became the private 167-line
`path_boolean_predicate.rs`; `path_experiment.rs` remains 1,996 lines. The
predicate tree is boxed inside `LocationPath`, so paths that do not activate
this bounded feature do not inherit the recursive tree's enum size. The module
owns no runtime frame, XSLT instruction compiler, resource authority, host
policy, or public representation.

## Boundaries

This tranche does not admit arbitrary axes, path-to-path comparison, qualified
name tests, variables, functions other than the exact `not` composition, or a
general predicate grammar. It does not change result order, node identity,
diagnostics, cancellation, resource authority, or corpus data.

## Verification

- A focused path test distinguishes equality, existential inequality, empty
  descendant node-sets, negation, and boolean composition.
- The complete 3,173-case local measurement produced the counters above.
- Strict workspace Clippy passes before the full verification gate.
