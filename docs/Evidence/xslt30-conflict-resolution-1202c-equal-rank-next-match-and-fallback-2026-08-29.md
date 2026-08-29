# XSLT30 `conflict-resolution-1202c` Equal-Rank Next-Match and Fallback

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1202c`
- Stylesheet: `conflict-resolution-1202.xsl`
- Source: inline `conflict-resolution-12` environment
- Native dependency: `spec=XSLT30+`

## Selection and execution

ADR-0007 selects XSLT 3.0 as FastXSLT's reference semantics. Under that
profile, equal-priority applicable rules in one stylesheet module use the last
declaration. The private selector already retained declaration indexes, so
`xsl:next-match` can move from the selected later rule to the earlier rule at
the same priority before continuing to lower priorities.

The two `match="*" priority="3"` rules therefore execute as `(3b)(3a)`. The
complete chain is `(5)(4)(3b)(3a)(2)`. The priority-`2` instruction contains an
`xsl:fallback` child. Because FastXSLT supports `xsl:next-match`, compilation
recognizes but does not compile or execute that fallback content.

The stylesheet declares `version="2.0"`, but the case metadata selects it only
for XSLT 3.0 and later. This evidence follows the project's modern semantic
profile; it does not infer legacy compatibility from the lexical version.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1202c` | `<out>(5)(4)(3b)(3a)(2)</out>` | semantically equal | passed |

## Claim boundary

This evidence admits XSLT 3.0 use-last selection and same-rank `xsl:next-match`
within one stylesheet module, plus inert `xsl:fallback` content on the supported
instruction. It does not admit `xsl:mode/@on-multiple-match`, warning policy,
the XSLT 1.0/2.0 recover profile in `1202a`, the error profile in `1202b`, root
template duplicates, import/package precedence, or `xsl:apply-imports`.
