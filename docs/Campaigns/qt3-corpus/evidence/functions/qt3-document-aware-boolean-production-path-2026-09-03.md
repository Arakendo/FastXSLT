# QT3 Document-Aware Boolean Production Path

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Four selected cases from `fn/not.xml` and five from `fn/boolean.xml`.
- The unchanged QT3 `atomic` and `auction` source environments.

## Method and result

Each unchanged expression is compiled inside `xsl:value-of` into an owned
document-aware EBV form. Location paths are parsed during compilation rather
than on each invocation. Production execution uses the real QT3 context
document and is compared with both the native QT3 assertion and the existing
direct evaluator.

All nine comparisons pass. They cover empty and nonempty descendant selections,
root nodes mixed before or after atomic values, namespace-wildcard name tests,
and an atomic value followed by an empty node sequence. The atomic-first
multi-item case reports `FORG0006` through production execution. A descendant
namespace-wildcard sentinel also reaches this form through the host-facing
workbench engine.

Together with the 245 source-free cases, all 254 selected cases across the
complete `true`, `false`, `not`, and `boolean` denominators now execute through
production. This changes no conserved denominator; the four sets remain part of
the existing 1,441-case QT3 aggregate.

## Boundary

This evidence does not admit general sequence construction, arbitrary
document-aware scalar expressions, atomization, or a public XPath AST. The
compiled form is private and its retained capacity participates in the existing
prepared-engine accounting model.
