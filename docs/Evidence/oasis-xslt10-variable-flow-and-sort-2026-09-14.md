# OASIS XSLT 1.0 variable flow and sort -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XSLT 1.0 variables retain their typed value through numeric conversion,
focus-derived arithmetic, sorting, and template-argument paths without creating
parallel evaluators for each consumer?

## Implemented slice

The shared compiler/runtime now composes these already-bounded operations:

- untyped global `true()` and `false()` bindings retain boolean atomic identity;
- the binary numeric evaluator recognizes `number($variable)` and applies XSLT
  1.0 boolean-to-number conversion before exact-rational arithmetic;
- a local `position() + nonnegative-integer` binding retains the current focus
  position and performs checked host-size arithmetic;
- `xsl:sort select="$variable"` evaluates through the existing XSLT 1.0
  variable string-value conversion;
- a source-node variable may feed a relative template-argument path such as
  `$authors/last-name`, preserving document order and duplicate elimination.

Sorting a source-node variable with an explicit sort key also exposed an
invalid internal assumption: the non-sorted special dispatcher handled source
variable sequences, while the sorted path treated that selection as
unreachable. The sorted path now performs a typed source-node lookup and
returns an ordinary failure for non-source values instead of panicking.

## Corpus result

Unchanged Microsoft `Variables__78164` becomes exact. Its local variable is
used as a sort key over a multi-node selection and preserves stable input order
when all keys compare equal.

Unchanged Microsoft `BVTs_bvt095` crosses every initialization and execution
boundary exercised by this tranche. Its semantic content agrees, but the
result remains a visible XML comparison mismatch because FastXSLT's current
indentation places whitespace inside the constructed `names` elements while
the expected document places indentation between them. It receives no exact
compatibility credit.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,703 | 1,705 | +2 |
| Initialization failures | 1,432 | 1,430 | -2 |
| Executed successfully | 1,520 | 1,522 | +2 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,387 | 1,388 | +1 |
| XML comparison mismatches | 106 | 107 | +1 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,388 / 2,742 = 50.62%` for standard-operation cases and
`1,388 / 3,173 = 43.74%` for the complete catalog.

## Boundaries

The focus arithmetic slice admits only addition of a nonnegative integer to
`position()`. Variable-rooted argument paths require source nodes. Sort
variables use XSLT 1.0 string-value conversion and do not admit arbitrary sort
expressions. Exact-rational arithmetic continues to reject NaN and other
unsupported lexicals through its existing typed failure. This work adds no
cross-document lookup, temporary-tree path extension, ambient resource access,
or public expression representation.

## Verification

- Focused tests cover untyped boolean globals, boolean/string/integer numeric
  conversion, checked focus-position offsets, variable sort keys over multiple
  nodes, and source-variable template-argument paths.
- The full 3,173-case measurement records two newly executed cases, one exact
  result, one visible comparison mismatch, and no panic.
- Formatting, strict Clippy, all 947 active workspace tests, documentation,
  Markdown links, unsafe-surface checks, and corpus inventories pass through
  `scripts/verify.ps1`.
