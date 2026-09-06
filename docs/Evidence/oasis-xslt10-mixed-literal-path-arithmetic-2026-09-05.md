# OASIS XSLT 1.0 Mixed Literal/Path Arithmetic

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `43573da` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; eight doubt-annotated expected-result matches promoted and two later boundaries exposed |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Shared typed-plan extension

The private recursive numeric plan now owns exact-decimal literal leaves and a
unary-negation node in addition to typed location-path leaves. Parenthesized
subexpressions remain part of the same precedence-preserving tree. A narrow
token repair recognizes subtraction between numeric lexicals and between
closed/open grouped operands while continuing to treat hyphens inside NCNames
as name characters.

This extends one shared plan rather than introducing a legacy evaluator.
Compilation still selects XSLT 1.0 first-node or modern zero-or-one path
conversion, while literals and unary operations execute identically. Focused
controls cover mixed path/literal addition and multiplication, a negated
subexpression, exact decimal multiplication, grouped subtraction, and the
existing modern cardinality failure.

Variables, functions, general atomic values, comparisons, and arbitrary XPath
primary expressions remain outside this path/literal tree.

## Corpus result

The complete 3,173-case sweep changed nine initialization dispositions. Eight
reach expected-result matches; two reach later execution boundaries and remain
uncredited:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,084 | 1,093 | +9 |
| Initialization failures | 2,051 | 2,042 | -9 |
| Executed successfully | 883 | 891 | +8 |
| XML comparison passes | 800 | 808 | +8 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 202 | +1 |

The promoted unchanged identities are:

- `Lotus/math_math85#1`;
- `Lotus/select_select21#1`;
- `Lotus/select_select22#1`;
- `Lotus/select_select23#1`;
- `Lotus/select_select29#1`;
- `Lotus/select_select36#1`;
- `Lotus/select_select39#1`; and
- `Lotus/select_select40#1`.

The seven `select` cases cover child, wildcard, attribute, and context paths
combined with integer literals using addition, subtraction, multiplication,
and division. `math85` composes nested precedence groups, literals, source
paths, and unary negation. Every promoted identity occurs in the suite's doubts
file, so FastXSLT retains that qualification.

The broader plan initially intercepted the existing compile-time XSLT 1.0
`0 div 0` rule. The final compiler order preserves that rule before admitting
general exact-rational arithmetic; unchanged `Lotus/boolean_boolean38#1`
therefore remains a passing `NaN` sentinel rather than becoming a runtime
zero-divisor failure.

The measured standard-operation ratio becomes 808 of 2,742 cases, or 29.47%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
