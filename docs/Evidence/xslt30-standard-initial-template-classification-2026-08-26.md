# XSLT30 Standard Initial-Template Classification

Date: 2026-08-26

## Question

Can the admitted source-free `castable-007` through `castable-009` cases cross
the harness boundary far enough to identify their first actual engine gap?

## Pinned inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/castable/_castable-test-set.xml`
- Cases: `castable-007`, `castable-008`, and `castable-009`
- Entry metadata: no explicit `initial-template`, which selects the standard
  initial template
- Result metadata: one `all-of` containing two XPath `assert` children per case

## Implemented boundary

The private compiler now resolves the lexical QName on
`xsl:template/@name` before storing the reserved standard initial-template
identity. A prefix bound to the XSLT namespace and local name
`initial-template` is stored as
`Q{http://www.w3.org/1999/XSL/Transform}initial-template`. Ordinary unprefixed
named templates retain their existing identity. Unsupported prefixed names do
not acquire accidental lexical-prefix semantics.

The corpus admission test now derives the absent entry declaration, inventories
both compound assertion children, admits each stylesheet into the sealed
resource snapshot, and compiles it. All three cases cross the entry/harness
boundary and produce the same structured engine disposition:

| Cases | Selection | Execution | First engine gap |
| --- | --- | --- | --- |
| `castable-007` through `castable-009` | selected | engine unsupported | `FXST1002`: top-level `xsl:function` |

This changes the complete nine-case denominator to seven selected and two
profile-excluded; the selected cases comprise four passes and three engine
gaps, with no harness gaps or unaccounted cases.

## Claim boundary

This evidence establishes namespace-stable recognition of the reserved entry
name and honest classification of the three native cases. It does not establish
general QName-valued declaration support, user-defined functions, dynamic date
context, predicate semantics, sequence-type list casts, expand-text value
templates, or evaluation of arbitrary XPath assertions.

The next executable pressure is no longer harness construction. It is a choice
of coherent engine semantics: either user-defined function calls and their
dynamic context, or another complete test-set family whose selected cases use
the already admitted expression model.
