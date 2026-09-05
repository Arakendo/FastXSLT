# OASIS XSLT 1.0 Ordered Literal Comparison

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `2d5df4e` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; one doubt-annotated expected-result match promoted |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Compatibility seam

XPath 1.0 ordered comparisons convert source-free string and number operands to
numbers. Modern XPath instead retains typed comparison rules, so blindly
adding ordered string comparison to the shared literal evaluator would select
the wrong meaning for cases such as `'10' > '2'`.

The private compile-selected compatibility hook now recognizes `<`, `<=`, `>`,
and `>=` when both operands are source-free string or finite numeric literals
and at least one operand is a string. It applies XPath 1.0 numeric conversion,
including `NaN` behavior for an invalid numeric string, and lowers the result to
the existing typed boolean constant. Numeric/numeric comparison remains owned
by the shared evaluator. The equivalent mixed-type version 3.0 stylesheet stays
rejected, and runtime contains no version branch.

Focused coverage includes numeric-looking strings whose lexical and numeric
orders differ, both string/number operand directions, invalid numeric strings,
and refusal to claim numeric/numeric or source-dependent operands.

## Corpus result

The complete 3,173-case sweep changed only the intended frontier:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,060 | 1,061 | +1 |
| Initialization failures | 2,075 | 2,074 | -1 |
| Executed successfully | 859 | 860 | +1 |
| XML comparison passes | 776 | 777 | +1 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 201 | 0 |

The promoted unchanged case is `Lotus/boolean_boolean49#1`, whose expression
`'2' > '1'` produces `true`. The identity occurs in the suite's doubts file, so
FastXSLT retains that qualification rather than presenting it as unqualified
conformance evidence.

The measured standard-operation ratio becomes 777 of 2,742 cases, or 28.34%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
