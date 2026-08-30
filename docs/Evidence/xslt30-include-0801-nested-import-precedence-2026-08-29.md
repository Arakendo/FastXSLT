# XSLT30 `include-0801` Nested-Import Precedence

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0801`
- Principal: `include-0801.xsl`
- Imported branches: `include-0701b.xsl`, `include-0701c.xsl`
- Imported leaves: `include-0701d.xsl`, `include-0701e.xsl`
- Source/result: `include-08.xml`, `include-0801.out`

## Assembly

All five stylesheet resources and the source are admitted before compilation.
The loader retains the two ordered principal imports and each branch's single
leaf import. The compiler first assembles each branch, then rebases the two
subtrees into distinct precedence ranges below the principal:

| Stratum | Module | Precedence |
| --- | --- | ---: |
| first leaf | `include-0701d.xsl` | -4 |
| first branch | `include-0701b.xsl` | -3 |
| second leaf | `include-0701e.xsl` | -2 |
| second branch | `include-0701c.xsl` | -1 |
| principal | `include-0801.xsl` | 0 |

The compiled matched-template sequence conserves the exact precedence vector
`[-4, -4, -3, -2, -2, -1, -1, 0]`. The principal `title` rule's
`xsl:apply-imports` therefore reaches the later `C-title` branch rule, whose own
`xsl:apply-imports` reaches `E-title`. The remaining author, overview, chapters,
and chapter rules exercise both imported subtrees and built-in fallback.

## Result and boundary

The serialized result is XML-equivalent to the pinned file-backed assertion.
This completes execution of all 13 positive cases in the 16-case include test
set. The two DTD-dependent cases and one expected multiple-match error remain
explicit non-passes.

This evidence admits one exact depth-two, five-module import topology. It does
not admit arbitrary recursive graphs, more than two sibling imports, repeated
module identities, configurable public dependency limits, or general
named-template/global-binding conflict resolution across nested import trees.
