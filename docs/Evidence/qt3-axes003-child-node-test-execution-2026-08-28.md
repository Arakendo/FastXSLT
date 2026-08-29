# QT3 `Axes003` Child-Node-Test Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: `Axes003-1` through `Axes003-4`
- Expression: `fn:count(//center/child::node())`
- Environments: `TreeTrunc`, `Tree1Text`, `Tree1Child`, and `TreeRepeat`
- Native `assert-eq` expectations: `0`, `1`, `1`, and `19`

## Method

The shared metadata-driven axis test resolves every case, referenced
environment, source, expression, and assertion from the pinned QT3 test set.
Each source is imported into a bounded sealed snapshot and built into owned XDM
before the expression executes through the direct XPath `fn:count` seam.

The private path representation now distinguishes named element tests, the
any-element wildcard, and the any-child-node test as typed variants. The
`child::node()` variant accepts every node in the XDM child sequence in document
order and never inspects attributes as children. Each examined child is charged
once to the invocation-local XPath node-visit domain.

A focused control selects text, element, processing-instruction, and comment
children in order and verifies four corresponding node-visit charges.

## Result

| Case | Environment | Expected | Actual | Disposition |
| --- | --- | ---: | ---: | --- |
| `Axes003-1` | `TreeTrunc` | 0 | 0 | passed |
| `Axes003-2` | `Tree1Text` | 1 | 1 | passed |
| `Axes003-3` | `Tree1Child` | 1 | 1 | passed |
| `Axes003-4` | `TreeRepeat` | 19 | 19 | passed |

The group is four selected, four passed, zero failed, zero unsupported, and
zero harness errors. Together, complete `Axes001` through `Axes003` contribute
eleven passing child-axis cases without an XSLT wrapper.

## Claim boundary

This evidence admits only `node()` as the node test of an explicit child-axis
step in the private path grammar. It does not admit `text()`, `comment()`,
processing-instruction tests with or without targets, namespace tests,
attribute axes, other axes, or general XPath sequence typing.
