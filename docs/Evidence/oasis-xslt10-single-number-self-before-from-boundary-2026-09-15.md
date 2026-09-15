# OASIS XSLT 1.0 Single Number Self Before `from` Boundary -- 2026-09-15

Date: 2026-09-15  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

For `xsl:number level="single"`, may a context node that itself matches the
effective `count` pattern be numbered when no ancestor matches the optional
`from` pattern?

## Implemented slice

Yes. The single-level number executor now distinguishes the context node from
a counted ancestor. A context node that matches `count` remains eligible for
sibling-position numbering even when no `from` ancestor exists. A counted
ancestor still requires the requested `from` boundary, preserving the existing
empty-result behavior when that boundary is absent.

The change remains in the shared controlled number executor. Ancestor and
sibling inspection retain their existing work charges; no alternate numbering
path or corpus-specific special case was introduced.

## Corpus result

The unchanged Lotus `numbering_numbering20` case now produces all expected
numbers. Its top-level `note` elements count from 1 through 9 even though they
have no `chapter` ancestor, while notes inside each chapter restart at 1 and
count through 3.

The case becomes an exact XML-comparison pass. The comparator's existing XML
line-ending normalization accounts for the upstream CRLF serialization without
discarding meaningful result-tree whitespace.

The same rule moves Microsoft `Number__84687` from pass to visible mismatch.
That case's suite-owned doubt metadata identifies this exact special `from`
behavior as a gray area, selects the older empty-number behavior, and says the
question was referred to the working group for an erratum. FastXSLT does not
retain that disputed behavior merely to preserve the aggregate count.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,771 | 1,771 | 0 |
| Initialization failures | 1,364 | 1,364 | 0 |
| Executed successfully | 1,587 | 1,587 | 0 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,444 | 1,444 | 0 |
| XML comparison mismatches | 114 | 114 | 0 |
| Execution panics | 0 | 0 | 0 |

The conserved pass and mismatch totals therefore hide a deliberate identity
exchange: unqualified Lotus `numbering_numbering20` moves from mismatch to pass,
while doubts-annotated Microsoft `Number__84687` moves from pass to mismatch.

The exact compatibility lower bound remains
`1,444 / 2,742 = 52.66%` for standard-operation cases and
`1,444 / 3,173 = 45.51%` for the complete catalog.

## Verification

Focused runtime coverage proves both sides of the boundary: self-matching
context nodes number without a `from` ancestor, while a counted ancestor still
produces no number without its required boundary. The unchanged Lotus corpus
case verifies top-level continuation and chapter-local restarts. An identity
comparison against the preceding commit proves the exact mismatch exchange.
The complete local gate set verifies the broader engine and documentation
invariants.
