# OASIS XSLT 1.0 path-to-path `contains()` -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing XSLT 1.0 path string-function plan apply first-node
string-value conversion to both arguments of `contains()` without creating a
second evaluator?

## Implemented slice

The private path string-function representation now retains its second operand
as either a literal or a typed location path. The compiler admits a path second
operand only for XSLT 1.0 `contains()` and applies the same static default-
namespace resolution used for the first path. Runtime evaluation obtains the
first selected node's controlled string value independently for both paths,
charges the existing string operation, and emits the existing boolean result.

Prepared-state accounting includes either the literal capacity or the complete
second path's known owned capacity. Literal `contains`, `starts-with`,
`substring-before`, and `substring-after` behavior continues through the same
representation and evaluator.

## Corpus result

Unchanged Lotus `string_string125` through `string_string128` all move from the
generic function-shaped path frontier to exact expected-result matches. The
cases cover true and false containment and multi-node operands where XPath 1.0
conversion must use only the first node in document order.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,714 | 1,718 | +4 |
| Initialization failures | 1,421 | 1,417 | -4 |
| Executed successfully | 1,531 | 1,535 | +4 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,398 | 1,402 | +4 |
| XML comparison mismatches | 106 | 106 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,402 / 2,742 = 51.13%` for standard-operation cases and
`1,402 / 3,173 = 44.19%` for the complete catalog.

## Boundaries

This slice adds only path-to-path `contains()` in XSLT 1.0 compatibility mode.
Path operands for other binary string functions, general function arguments,
variables, temporary-tree navigation, and modern sequence-aware function
conversion remain outside this change. Both paths remain source-document paths
and use the existing bounded evaluator and work control.

## Verification

- A focused transformation proves true and false path-to-path containment and
  first-node conversion when either path selects multiple nodes.
- Existing literal path string-function cases remain in the same focused test.
- The full 3,173-case sweep moves all four affected cases to exact results
  without a mismatch, execution failure, or panic.
- Formatting, strict Clippy, all workspace tests, documentation, Markdown
  links, unsafe-surface checks, and corpus inventories pass through
  `scripts/verify.ps1`.
