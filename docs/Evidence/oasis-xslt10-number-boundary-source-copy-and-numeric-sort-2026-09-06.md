# OASIS XSLT 1.0 Number Boundary, Source Copy, and Numeric Sort Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Corpus | OASIS XSLT/XPath 1.0 CD04, locally acquired and hash-verified |
| Scope | `xsl:number` single-level `from` boundaries, one source-copy attribute expression, and XSLT 1.0 numeric-sort conversion |
| Disposition | Three comparison mismatches become exact expected-result matches; one additional mismatch becomes semantically correct but remains uncredited because of discretionary serialization spelling |

## Changes

Single-level numbering now allows one ancestor to satisfy both `count` and
`from`, while still requiring a specified `from` boundary to exist. This fixes
the level-A values in `Lotus/numbering_numbering63#1` and prevents numbering
from leaking into a sibling subtree in `Microsoft/Number__84687#1`. The latter
case becomes exact; the Lotus result now differs only through a newly exposed,
upstream-doubted `numbering20` comparison elsewhere in the measured frontier.

The private source-element `xsl:copy` compiler now retains the exact form
`xsl:attribute` containing one `xsl:value-of select="@unqualified-name"` as a
source-derived value. Execution reads the current source element's matching
attribute, charges each visited source attribute, and produces an empty string
when none exists. `Microsoft/Sorting__84186#1` consequently carries the correct
`a`, `1`, `2`, and `3` values. It remains an XML comparison mismatch only
because the expected artifact fixes discretionary indentation and empty-element
spelling; no exact pass is claimed for it.

Numeric `xsl:sort` now retains whether its stylesheet selected XSLT 1.0
compatibility. That path uses the XPath 1.0 number lexical grammar, so a leading
plus sign becomes NaN rather than being accepted by Rust's `f64` parser. Numeric
comparison also treats positive and negative zero as equal and lets stable sort
preserve their input order. Modern sort conversion remains on its existing
path. The two Microsoft leading-plus and signed-zero cases become exact.

Focused tests preserve the same-node/absent `from` distinction, source-copy
attribute lookup including absence, and stable numeric ordering for leading-plus
and signed-zero values.

## Measurement

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,269 | 1,269 | 0 |
| Executed successfully | 1,096 | 1,096 | 0 |
| Expected-result XML matches | 983 | 986 | +3 |
| XML comparison mismatches | 78 | 75 | -3 |
| Doubt-annotated comparison mismatches | 9 | 8 | -1 |
| Execution failures | 173 | 173 | 0 |
| Expected execution errors observed | 15 | 15 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |

The strict standard-operation lower bound is now `986 / 2,742 = 35.96%`;
the deliberately conservative all-catalog ratio is `986 / 3,173 = 31.07%`.
This is compatibility evidence, not an XSLT 1.0 conformance claim.
