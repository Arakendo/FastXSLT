# OASIS XSLT 1.0 Static Template-Argument Tree -- 2026-09-15

Date: 2026-09-15  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a content-valued `xsl:with-param` construct a mixed text and literal-result
temporary tree without introducing a second runtime tree builder or claiming
general sequence-constructor support?

## Implemented slice

Yes. The private named-template argument compiler now recognizes a bounded
static content form made only from text and literal result elements. It reuses
the existing compile-owned `ConstructedNode` representation and static
constructed-child compiler. XSLT instruction children and dynamic literal
attributes remain rejected by this form.

At invocation time, the retained static plan is materialized as an
invocation-owned temporary tree with one fresh tree identity. Mixed root text
and element nodes preserve their order and structure, and existing temporary
tree copy semantics consume the result. Compiled retention accounting includes
the constructed plan. No resource authority, caching, cross-invocation state,
or public representation was added.

## Corpus result

Unchanged Lotus `copy_copy08` moves from `FXST1033` initialization failure to
an exact XML-semantic expected-result pass. Its parameter content
`Please <b>BOLD THIS</b> now.` is constructed as a temporary tree and copied by
the called template.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,777 | 1,778 | +1 |
| Initialization failures | 1,358 | 1,357 | -1 |
| Executed successfully | 1,593 | 1,594 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,449 | 1,450 | +1 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,450 / 2,742 = 52.88%` for standard-operation cases and
`1,450 / 3,173 = 45.70%` for the complete catalog.

## Verification

A focused production-lifecycle test passes mixed static content through a
named template and verifies the copied serialized structure. The unchanged
full sweep conserves all 3,173 identities and adds no failure, mismatch, or
panic.
