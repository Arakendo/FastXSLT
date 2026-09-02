# XSLT30 `mode-1107` streaming profile classification -- 2026-09-02

## Result

The unchanged W3C XSLT30 `mode-1107a`, `mode-1107b`, and `mode-1107c` cases
are now explicit profile exclusions rather than generic harness defaults. Each
native case supplies a principal source with `streaming="true"`, selects the
same stylesheet with its static `STREAMABLE` parameter set to true, and tests
accumulator behavior in a streamable mode.

ADR-0007 deliberately excludes XSLT streaming conformance. Executing these
cases through the ordinary tree evaluator would therefore create misleading
evidence even if the eventual result happened to match.

## Ledger invariant

The complete-denominator adapter now accepts streaming authority from either
the suite's explicit streaming feature dependency or its native streamed-source
metadata. Every listed streaming exclusion must preserve at least one of those
upstream signals; an arbitrary case name cannot silently enter the exclusion
set.

## Conservation

The complete 169-case `attr/mode` denominator remains at 86 passes, while
profile exclusions move from 45 to 48 and visible default not-run cases move
from 38 to 35. Across the eleven conserved XSLT30 denominators, the total
remains at 406 passes and three engine-unsupported cases; profile exclusions
move from 51 to 54 and visible default not-run cases move from 71 to 68.
