# OASIS XSLT 1.0 Number/String Equality Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `c394610` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; two unchanged standard cases promoted to definite passes |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Semantic slice

XPath 1.0 equality converts a string to a number when its other atomic operand
is numeric. The existing compile-selected XSLT 1.0 compatibility hook already
implemented the higher-precedence boolean conversion rule. This tranche extends
that same private hook to constant number/string `=` and `!=` comparisons.

The compatibility behavior is selected once from the stylesheet's `version`
during compilation. It lowers to the existing typed boolean constant and adds
no version branch to runtime evaluation. The identical expressions remain
outside the admitted modern expression slice rather than changing XPath 3.1
comparison semantics.

Focused coverage includes leading-zero numeric strings, surrounding XPath
whitespace, invalid numeric strings becoming `NaN`, both operand orders, and
equality/inequality. The executable lifecycle regression combines these cases
with the earlier boolean-dominant rules and proves that an equivalent version
3.0 stylesheet remains rejected.

## Corpus result

The complete 3,173-case sweep changed only the intended frontier:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,052 | 1,054 | +2 |
| Initialization failures | 2,083 | 2,081 | -2 |
| Executed successfully | 851 | 853 | +2 |
| XML comparison passes | 768 | 770 | +2 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 201 | 0 |

The promoted unchanged cases are:

- `Lotus/boolean_boolean14#1`: `1 = '001'`;
- `Lotus/boolean_boolean81#1`: `'001' = 1`.

The strict standard-operation lower bound is therefore 770 of 2,742 cases, or
28.08%. This is compatibility evidence, not an XSLT 1.0 conformance claim.

## Harness repair

The measurement script previously exported an empty trace-filter environment
variable when `-TraceCase` was omitted. Because every identity contains the
empty string, a normal sweep emitted a trace for every executed case. The script
now removes that variable for an omitted or whitespace-only filter and retains
the prior environment afterward. A focused comparator regression also confirms
that XML-equivalent empty-element syntax and prolog spacing do not create false
mismatches. Neither repair changes case disposition.

