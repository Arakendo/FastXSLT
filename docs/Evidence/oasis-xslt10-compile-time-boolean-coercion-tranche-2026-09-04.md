# OASIS XSLT 1.0 Compile-Time Boolean-Coercion Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 762 definite unchanged XML passes; 1,046 initialized cases |
| Result | 768 definite unchanged XML passes; 1,052 initialized cases |
| Disposition | First compile-time compatibility-mode proof; not a conformance claim |

A private typed value-expression static context now derives an initial
compatibility mode from the containing `xsl:stylesheet` or `xsl:transform`
root. Under exactly `version="1.0"`, the compiler applies XPath 1.0's
boolean-dominant equality conversion to mixed source-free boolean/string or
boolean/finite-number literals. It retains only the resulting typed boolean in
the executable plan. The identical expression under `version="3.0"` remains
unsupported, proving that version selection occurs during compilation rather
than through a runtime branch or second evaluator.

The unchanged `Lotus/boolean_boolean15#1` through
`Lotus/boolean_boolean18#1`, `Lotus/boolean_boolean82#1`, and
`Lotus/boolean_boolean83#1` cases pass. Initialization moves from 1,046 to
1,052, execution from 845 to 851, and XML passes from 762 to 768. Mismatches
remain 50, comparator gaps 19, execution failures 201, unexpected successes
14, and panics 0.

The strict lower bound is now **768 / 2,742 = 28.01%** of standard-operation
cases and **768 / 3,173 = 24.20%** of the complete catalog. Focused tests cover
operand order, string and numeric effective-boolean conversion, equality and
inequality results, rejection of non-boolean mixed equality, and paired legacy
versus modern stylesheet compilation. General mixed number/string comparison,
path operands, local version declarations, special numeric values, and broader
backwards-compatible behavior remain explicit future work.
