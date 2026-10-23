# OASIS XSLT 1.0 Number Positional and Parent/Child Pattern Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Corpus | OASIS XSLT/XPath 1.0 CD04, locally acquired and hash-verified |
| Scope | Bounded sibling-position predicates and one parent/child relationship in `xsl:number` patterns |
| Disposition | Three new expected-result matches; no new mismatch or execution failure |

## Change

The private typed number-pattern tree now admits exact positive sibling
positions and the static form `position() mod N = R`, where `N` is nonzero.
Position is computed among sibling elements matching the pattern's expanded
name, and every inspected sibling is charged. This preserves pattern predicate
focus rather than borrowing the current transformation focus.

The same tree admits one explicit parent/child relationship such as
`chapter/note` or `*/first-name[1]`. Matching first proves the child pattern,
then charges and tests the immediate parent. Descendant paths, multi-step
chains, `id()`, `key()`, and general predicates remain unsupported.

Focused production tests cover odd sibling positions, exact position two,
exact parent/child names, and a wildcard parent composed with a positional
child.

## Measurement

`scripts/measure-oasis-xslt10.ps1` changed the conserved measurement as follows:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,233 | 1,236 | +3 |
| Executed successfully | 1,022 | 1,025 | +3 |
| Expected-result XML matches | 912 | 915 | +3 |
| XML comparison mismatches | 76 | 76 | 0 |
| Execution failures | 211 | 211 | 0 |

The next visible number-pattern frontier is `id('b d g')`; it remains a
structured `FXST1050` unsupported outcome. These three passes are compatibility
evidence, not an XSLT 1.0 conformance claim.

