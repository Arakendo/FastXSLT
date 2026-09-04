# QT3 Source-Free Boolean Production Path

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Existing conserved `fn/true.xml`, `fn/false.xml`, `fn/not.xml`, and
  `fn/boolean.xml` denominators.
- 245 selected source-free cases: 24 true, 24 false, 74 not, and 123 boolean.

## Method and result

Each unchanged expression is parsed once into an owned production scalar AST,
compiled inside `xsl:value-of`, executed by the ordinary runtime, serialized,
and compared with its native QT3 assertion or diagnostic. The focused evaluator
continues to run beside it as a work-accounting oracle. All 245 production
comparisons pass. A composed `not`/`boolean` sentinel reaches the same path
through the host-facing workbench engine.

The production path preserves `XPST0017` for invalid arity and `FORG0006` where
the effective boolean value is undefined. This migration changes no conserved
denominator: the four sets remain part of the existing 1,441-case QT3 aggregate.

## Boundary

The nine selected document-aware cases—four from `not` and five from
`boolean`—still execute only their dedicated context evaluator and are not
counted here. This evidence does not admit arbitrary document focus, general
atomization, dynamic sequences, or a public scalar-expression representation.
