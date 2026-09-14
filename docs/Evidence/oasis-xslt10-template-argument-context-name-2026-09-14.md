# OASIS XSLT 1.0 context-name template argument -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a named-template argument reuse the caller's source-node lexical name
without adding a general function-expression evaluator or losing namespace
prefix information?

## Implemented slice

The template-invocation compiler now recognizes the exact `name()` and
`name(.)` argument forms and retains a typed context-node-name argument. At
invocation time it requires a source-node focus, charges the XPath operation
and node visit, and derives the string through the existing source lexical-name
semantics. A prefixed source element therefore supplies its lexical name, such
as `p:doc`, rather than only its expanded local name.

This is a bounded reuse of an already-supported context operation. It does not
admit arbitrary function calls in template arguments or introduce another name
resolution path.

## Corpus result

Unchanged Lotus `namedtemplate_namedtemplate03` leaves `FXXP1011` and becomes
an exact result. Its recursive named-template calls pass `name(.)` from the
current source element while retaining the existing context for each call.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,702 | 1,703 | +1 |
| Initialization failures | 1,433 | 1,432 | -1 |
| Executed successfully | 1,519 | 1,520 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,386 | 1,387 | +1 |
| XML comparison mismatches | 106 | 106 | 0 |

The exact compatibility lower bound is now
`1,387 / 2,742 = 50.58%` for standard-operation cases and
`1,387 / 3,173 = 43.71%` for the complete catalog.

## Boundaries

The argument requires a source context node. Missing or non-source context
remains a typed dynamic failure. This slice does not add temporary-tree name
semantics, arbitrary context functions, dynamic function dispatch, new resource
authority, or a public expression representation.

## Verification

- A focused runtime test proves that `name(.)` passed from a prefixed source
  element retains `p:doc` across a named-template call.
- The complete 3,173-case measurement moves exactly one case from
  initialization failure to exact result without a new mismatch or execution
  failure.
- Formatting, strict Clippy, the complete workspace suite, documentation, link,
  unsafe-surface, and corpus-inventory checks pass through `scripts/verify.ps1`.
