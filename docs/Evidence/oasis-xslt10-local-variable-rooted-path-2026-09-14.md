# OASIS XSLT 1.0 local variable-rooted path -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a local source-node variable feed a relative path into another local
node-set variable while preserving node identity, document order, focus, and
bounded XPath evaluation?

## Implemented slice

The existing XSLT 1.0 variable-rooted path compiler is now reused for local
variable bindings such as `$store/book`. The retained private instruction owns
the source variable name and typed relative `LocationPath`. Runtime evaluation
requires the source binding to contain source nodes, evaluates the relative path
from every bound root through the existing controlled evaluator, then restores
document order and removes duplicate identities before binding the result.

This is the same selection behavior already used by apply-templates and value
consumers; it does not introduce a second path evaluator. The compiler/runtime
dispatchers were decomposed through focused helpers while adding the new
binding, keeping their enforced source-unit limits intact.

## Corpus result

`Microsoft/XSLTFunctions_LastFuncAppliedToNodesetBoundToVariable` leaves
`FXXP1008` and becomes an exact result. Its `$bookstore/book` binding contains
six nodes, and a subsequent `xsl:for-each` observes `last() = 6` for every item.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,701 | 1,702 | +1 |
| Initialization failures | 1,434 | 1,433 | -1 |
| Executed successfully | 1,518 | 1,519 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,385 | 1,386 | +1 |
| XML comparison mismatches | 106 | 106 | 0 |
| `FXXP1008` initialization frontier | 6 | 5 | -1 |

The exact compatibility lower bound is now
`1,386 / 2,742 = 50.55%` for standard-operation cases and
`1,386 / 3,173 = 43.68%` for the complete catalog.

## Boundaries

The source operand must already be a source-node sequence. Atomic and temporary
tree operands remain typed failures rather than being silently coerced. This
does not add cross-document composition, `document()`, `current()`, arbitrary
variable expressions, cache retention, or a public node-set representation.

## Verification

- A focused runtime test binds a three-node `$store/book` sequence and proves
  the derived focus size through `last()`.
- The complete 3,173-case measurement moves exactly one case from
  initialization failure to exact result without a new mismatch or execution
  failure.
- Formatting, strict Clippy, the complete workspace suite, documentation, link,
  unsafe-surface, and corpus-inventory checks pass through `scripts/verify.ps1`.
