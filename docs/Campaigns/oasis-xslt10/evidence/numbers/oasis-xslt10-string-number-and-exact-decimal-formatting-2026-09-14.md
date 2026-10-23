# OASIS XSLT 1.0 nested numeric string conversion -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a bounded XSLT 1.0 `string()` wrapper reuse the existing numeric plans, and
can the shared exact-decimal formatter represent the long finite decimals that
the unchanged corpus case exposes?

## Implemented slice

While compiling a version 1.0 stylesheet, the value-expression compiler now
unwraps `string(number(path))` and a narrowly admitted product of one numeric
literal with `number(path)`. Both forms lower to the existing controlled numeric
plans because `xsl:value-of` observes the same string result. Other nested
numeric expressions remain outside the compatibility grammar, and the
equivalent version 3.0 form remains unsupported by this slice.

The corpus probe exposed an independent shared defect in exact-decimal
formatting. A terminating rational denominator of `2^a * 5^b` previously used
`a + b` decimal places and compensated with a large power of ten. That happened
to trim back to the right small answers, but could overflow while formatting a
valid long decimal. Formatting now uses the minimal decimal scale
`max(a, b)` and multiplies only by the missing powers of two and five. Focused
tests retain existing decimals and add 28-place values.

## Corpus result

Unchanged Lotus `math_math111` moves from initialization failure to an exact
expected result. It exercises eleven positive and negative conversions,
including two 28-place decimals.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,726 | 1,727 | +1 |
| Initialization failures | 1,409 | 1,408 | -1 |
| Executed successfully | 1,543 | 1,544 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,410 | 1,411 | +1 |
| XML comparison mismatches | 106 | 106 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,411 / 2,742 = 51.46%` for standard-operation cases and
`1,411 / 3,173 = 44.47%` for the complete catalog.

## Boundaries and verification

The nested wrapper admits only `number(path)` or a multiplication pairing one
numeric literal with `number(path)`. It does not admit arbitrary arithmetic,
variables, division-by-zero behavior, or a general nested function grammar.
Two nearby corpus expressions that would otherwise reach unsupported runtime
division remain initialization failures, so the tranche adds no execution
failure.

Focused tests cover exact-decimal formatting and the version-sensitive
compile/execute lifecycle. The complete corpus sweep verifies the intended one-
case movement with no mismatch, execution failure, or panic. The complete
`scripts/verify.ps1` gate passes: unsafe-surface policy, formatting, strict
Clippy, 918 active engine tests, workbench and worker tests, documentation,
Markdown links, and pinned-corpus integrity.
