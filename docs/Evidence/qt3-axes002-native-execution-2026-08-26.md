# QT3 `Axes002` Native Execution

Date: 2026-08-26

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: `Axes002-1` through `Axes002-4`
- Expression: `fn:count(//center/child::south-east)`
- Environments: `TreeTrunc`, `Tree1Child`, `TreeCompass`, and `TreeRepeat`
- Native `assert-eq` expectations: `0`, `0`, `1`, and `2`

## Method

The metadata-driven Rust test resolves each selected case, referenced
environment, source file, expression, and assertion from the pinned QT3 test
set. It reads the source through the import adapter, closes the file handle,
admits the bytes into a one-entry bounded resource set, seals the snapshot, and
constructs the owned XDM document from snapshot bytes.

The expression then executes directly through the private XPath path. Leading
descendant navigation finds each `center`, an explicit named `child::` step
selects `south-east`, and the narrow `fn:count` seam returns the sequence size.
The harness compares that integer with the native QT3 `assert-eq` value. Every
examined path node is charged to the invocation's XPath node-visit work domain.
No XSLT stylesheet wrapper is involved.

## Result

| Case | Environment | Expected | Actual | Disposition |
| --- | --- | ---: | ---: | --- |
| `Axes002-1` | `TreeTrunc` | 0 | 0 | passed |
| `Axes002-2` | `Tree1Child` | 0 | 0 | passed |
| `Axes002-3` | `TreeCompass` | 1 | 1 | passed |
| `Axes002-4` | `TreeRepeat` | 2 | 2 | passed |

The conserved denominator is four selected, four passed, zero failed, zero
unsupported, and zero harness errors. The adjacent selected `Axes001-1` case
remains explicitly engine-unsupported because its `child::*` wildcard exceeds
the implemented path grammar.

## Claim boundary

This evidence establishes native execution only for the exact selected
named-child-axis group and its metadata shapes. The implementation recognizes a
narrow `fn:count(...)` wrapper around the existing child-path grammar; it is not
a general function library. Wildcards, other axis steps, general sequences,
broader QT3 assertion families, and XPath conformance remain outside the claim.
