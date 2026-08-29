# QT3 `Axes001` Element-Wildcard Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: `Axes001-1` through `Axes001-3`
- Expression: `fn:count(//center/child::*)`
- Environments: `TreeTrunc`, `Tree1Child`, and `TreeRepeat`
- Native `assert-eq` expectations: `0`, `1`, and `6`

## Method

The existing metadata-driven axis test resolves each case, environment, source
file, expression, and assertion from the pinned QT3 set. Source bytes are read
by the import adapter, admitted into a bounded resource set, sealed, and parsed
into owned XDM before direct XPath evaluation.

The path parser now retains `*` as an element name test on a child step. During
evaluation it selects every child whose node kind is element, independent of
namespace, while excluding text, comment, and processing-instruction children.
Every examined child remains charged to the invocation-local XPath node-visit
domain. Named unprefixed steps retain their existing no-namespace semantics.

A focused control uses mixed text, comment, unnamespaced element, and namespaced
element children. It proves that both element children match and that all four
examined child nodes are charged even though only two enter the result.

## Result

| Case | Environment | Expected | Actual | Disposition |
| --- | --- | ---: | ---: | --- |
| `Axes001-1` | `TreeTrunc` | 0 | 0 | passed |
| `Axes001-2` | `Tree1Child` | 1 | 1 | passed |
| `Axes001-3` | `TreeRepeat` | 6 | 6 | passed |

The `Axes001` denominator is three selected, three passed, zero failed, zero
unsupported, and zero harness errors. Together with the adjacent `Axes002`
group, seven pinned element-child-axis cases execute through the same direct
XPath seam. Subsequent [`Axes003` evidence](qt3-axes003-child-node-test-execution-2026-08-28.md)
extends that seam to the four-case `child::node()` group without changing this
result.

## Claim boundary

This establishes only the element wildcard `child::*` in the admitted child
path grammar. It does not admit kind tests such as `child::node()`, namespace
wildcards, attribute wildcards, other axes, general sequence types, or broad
XPath conformance.
