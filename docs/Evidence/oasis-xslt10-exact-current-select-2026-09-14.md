# OASIS XSLT 1.0 exact `current()` select -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the exact XSLT 1.0 `current()` expression reuse the typed context-item
path without admitting the substantially broader semantics of `current()`
inside predicates and composed expressions?

## Implemented slice

The XSLT 1.0 compatibility path parser now lowers an expression whose complete
trimmed spelling is `current()` to the existing context-item origin. The value
expression compiler now consistently selects that compatibility parser for
XSLT 1.0 location paths instead of bypassing it on the final fallback path.

Because each instruction evaluates its own compiled expression against its
supplied dynamic context, exact `current()` observes the correct current node
inside nested `xsl:for-each` instructions. The typed path evaluator, work
accounting, first-node string conversion, and node-set behavior remain shared.

## Corpus result

All eight cases whose first frontier was exact `current()` now initialize and
execute. Unchanged Lotus `select_select02`, Lotus `select_select70`, and
Microsoft `XSLTFunctions__84175` become exact expected-result matches. Five
related Microsoft cases execute to visible comparison mismatches and receive no
exact compatibility credit.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,706 | 1,714 | +8 |
| Initialization failures | 1,429 | 1,421 | -8 |
| Executed successfully | 1,523 | 1,531 | +8 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,395 | 1,398 | +3 |
| XML comparison mismatches | 101 | 106 | +5 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,398 / 2,742 = 50.98%` for standard-operation cases and
`1,398 / 3,173 = 44.06%` for the complete catalog.

## Boundaries

Only an entire XSLT 1.0 expression equal to `current()` is admitted here.
`current()` nested in predicates, arithmetic, comparisons, function arguments,
or path composition continues to require its existing dedicated plans or
remains unsupported. Modern XPath location-path parsing is unchanged, and this
work adds no public expression representation or alternate evaluator.

## Verification

- A focused parser test proves exact `current()` and the context-item path
  compile identically only through the XSLT 1.0 compatibility entry point.
- A focused transformation nests `current()` in two `xsl:for-each`
  instructions and proves that each expression observes its instruction's
  current source node.
- The full 3,173-case sweep moves eight cases through initialization and
  execution, records three exact results and five visible mismatches, and
  records no panic.
- Formatting, strict Clippy, all workspace tests, documentation, Markdown
  links, unsafe-surface checks, and corpus inventories pass through
  `scripts/verify.ps1`.
