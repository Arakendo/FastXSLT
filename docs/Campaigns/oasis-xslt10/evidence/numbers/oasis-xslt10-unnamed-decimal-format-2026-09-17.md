# OASIS XSLT 1.0 Unnamed Decimal Format -- 2026-09-17

Date: 2026-09-17  
Status: Verified bounded semantic slice and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

Fifty-seven unchanged cases initially stopped at the blanket unsupported
`xsl:decimal-format` declaration boundary. Inspection showed that this was not
a safe declaration-only feature: the cases exercise customized picture
characters, non-finite labels, named formats, duplicate declarations, and
invalid symbol combinations.

The bounded coherent tranche was the unnamed decimal format used by two-
argument `format-number()`. Its values are stylesheet-static and can be copied
into the already compiled formatting expression rather than consulted through
a runtime registry or version branch.

## Implementation

The compiler now validates an unnamed `xsl:decimal-format`, composes repeated
non-conflicting property declarations, rejects conflicting properties as
`XTSE1290`, requires active formatting characters to be distinct, and copies
the resolved format into each compiled `format-number()` expression. The
evaluator applies configured decimal, grouping, minus, percent, per-mille,
digit, pattern-separator, infinity, and NaN values.

The first corpus pass exposed the negative-subpicture edge where identical
positive and negative subpictures still require the configured minus sign.
That rule is now covered by a focused evaluator test and the unchanged corpus
case.

This tranche does not admit named decimal formats. The subsequent
[static named-format tranche](oasis-xslt10-static-named-decimal-format-2026-09-17.md)
adds compile-resolved literal QName selection, followed by the
[digit-family tranche](oasis-xslt10-decimal-digit-family-2026-09-17.md).
Computed or dynamic third-argument format selection is not approximated.
General decimal-format composition across separately compiled include/import
modules remains outside this tranche; no cross-module precedence claim is
made.

## Corpus result

Relative to the preceding namespace-alias baseline:

- initialized cases rise from 1,903 to 1,923;
- executed-successfully cases rise from 1,699 to 1,715;
- exact expected-result matches rise from 1,533 to 1,549;
- initialization failures fall from 1,232 to 1,212;
- execution failures rise from 204 to 208 as four cases reach later explicit
  runtime boundaries;
- visible XML mismatches remain 137 after the one newly exposed formatting
  mismatch was repaired; and
- comparator-unsupported outcomes remain 23.

The remaining original declaration family is now divided into named-format,
non-ASCII digit-family, invalid/conflicting declaration, misplaced instruction,
and later execution frontiers rather than one blanket unsupported category.
Expected-error accounting remains 403 initialization observations, 21
execution observations, and four doubt-annotated unexpected successes. No
corpus panic occurs.

## Verification

Focused tests cover customized separators and labels, the identical-negative-
subpicture minus rule, named-format non-admission, duplicate-property conflict,
and non-distinct active symbols. The complete local measurement conserves all
3,173 catalog identities: 1,923 initialize, 1,715 execute successfully, 1,549
compare exactly, 137 remain visible mismatches, and 23 reach comparator-
unsupported outcomes.
