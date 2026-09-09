# OASIS XSLT 1.0 predicate cardinality and attribute length -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the private typed predicate path support bounded child cardinality,
attribute string length, and attribute inequality while reusing the ordinary
path evaluator and instruction condition effective boolean value?

## Decision in the experiment

The predicate tree now retains typed leaves for:

- `count(./name) = integer` over unprefixed child elements;
- `string-length(@name) = integer` and `> integer`;
- `@name != literal`, including the XSLT 1.0 empty-node-set behavior; and
- `not(...)` composition around an otherwise admitted leaf.

Instruction conditions may now treat an exactly parsed
`following-sibling::name` path as node existence. An exploratory admission of
all bare axis paths initialized one unrelated case with a wrong result, so the
production seam was narrowed to the one evidenced axis family before final
measurement.

Every examined child or attribute is charged as an XPath node visit. String
length is charged as an XPath operation. Missing attributes convert to the
empty string for the bounded string-length forms, while `@name != literal`
remains false when the node-set is empty.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,610 | 1,612 | +2 |
| Executed successfully | 1,439 | 1,441 | +2 |
| Expected-result XML matches | 1,319 | 1,321 | +2 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate57#1` and
`Lotus/predicate_predicate58#1` cases now match exactly. The strict
standard-operation lower bound is now `1,321 / 2,742 = 48.18%`; the
conservative all-catalog ratio is `1,321 / 3,173 = 41.63%`. These remain local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit general `count()` operands, general string-length
expressions, arbitrary relational operators, qualified names in these leaves,
or every bare-axis condition. It does not introduce a second conditional
evaluator or change node identity, result order, diagnostics, cancellation,
resource authority, or corpus data.

## Verification

- A focused path test covers present, empty, nonempty, and missing attributes
  together with child counts.
- Traced unchanged cases prove the initially exposed later blockers were
  removed and both complete results match expected XML.
- The complete 3,173-case local measurement produced the counters above.
