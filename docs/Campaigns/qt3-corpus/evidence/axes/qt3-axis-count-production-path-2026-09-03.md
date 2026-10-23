# QT3 Axis Count Production Path

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- 182 selected successful cases from `prod/AxisStep.xml`.
- The unchanged QT3 source environments and `fn:count(...)` expressions.

## Method and result

Production compilation now accepts the standard predeclared `fn:count`
spelling as well as the already admitted unprefixed spelling. Each unchanged
expression is compiled inside `xsl:value-of`, executes the owned location path
against its real QT3 context document, and serializes the numeric result. The
result is compared with the native QT3 assertion and the existing direct count
evaluator.

All 182 comparisons pass. A prefixed-count sentinel also reaches the same
compiler and runtime through the host-facing workbench engine. This raises the
production-backed selected QT3 subtotal from 946 to 1,128 without changing the
conserved 1,441-case aggregate.

## Boundary

This evidence admits no new axis or path semantics beyond the existing typed
location-path implementation. It establishes that the selected successful
`AxisStep` paths are reachable through production compilation, execution, and
serialization. The remaining 42 selected cases in that test set exercise empty
path operands and static or dynamic diagnostics and remain separate work.
