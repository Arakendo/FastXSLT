# OASIS XSLT 1.0 numeric attribute predicate evidence

Date: 2026-09-30

## Question

Can numeric attribute comparisons such as `@size > 17` use one bounded typed
predicate in ordinary location paths and named XSLT 1.0 match patterns without
creating separate source and temporary-tree semantics?

## Implementation

The location-path predicate grammar now admits an unqualified attribute on
either side of the six integer comparison operators. Evaluation visits the
candidate element's attributes under the existing XPath work controls, applies
the existing XPath number conversion, and treats a missing or nonnumeric value
as nonmatching.

The match compiler admits the same bounded numeric relation for a named element
pattern. The semantic plan retains the expanded element and attribute names,
integer operand, and relation. Source and temporary-tree selectors share the
same boolean predicate evaluator. The focused differential regression also
found and repaired an older temporary-tree focus assumption: a parentless
temporary root now derives its predicate position from the tree's root sequence
rather than being rejected before predicate evaluation.

No parser, DTD, XDM, resource authority, public API, or production-default DTD
behavior changed.

## Conserved OASIS result

The hash-verified 3,173-case OASIS CD04 sweep reports:

- 2,463 exact XML comparisons (77.62%), up from 2,460;
- 2,657 initialized cases, up from 2,654;
- 2,611 successful executions, up from 2,608;
- 513 initialization failures, down from 516;
- 46 execution failures, unchanged;
- ten visible mismatches and four comparator gaps, unchanged; and
- 423 / 431 expected-error credit, unchanged.

The three exact gains are:

- `Lotus/idkey_idkey37#1`;
- `Lotus/idkey_idkey38#1`; and
- `Lotus/idkey_idkey39#1`.

## Attribution observation

Successful compilation exposes three more principal-source internal subsets
that were previously hidden behind expression or match-pattern failures. The
observed direct-DTD attribution set grows from 126 to 129 cases: 27 bounded
internal-subset parses, 93 bounded sealed-external-subset parses, and nine
explicit unsupported declaration-semantics outcomes. This is improved
attribution, not a DTD profile expansion; AR-0025 remains deferred.
