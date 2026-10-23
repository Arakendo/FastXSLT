# XSLT30 `include-0601` Imported Simplified Root and Text Fallback

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0601`
- Principal stylesheet: `include-0601.xsl`
- Imported stylesheet: `include-0601a.xsl`
- Source: inline `<greeting>Hello world.</greeting>`

## Execution

The sealed snapshot contains the source and exactly two stylesheet modules. The
secondary module is a simplified stylesheet: its literal `html` document
element carries `xsl:version="2.0"` and acts as its implicit template.

During single-import assembly, FastXSLT compiles that simplified module and
normalizes its implicit template into an ordinary document-matching rule at
import precedence `-1`. The principal module's explicit `text()` rule remains
at precedence `0`. Initial source dispatch therefore selects the imported
document rule without granting it the principal direct-root fast path.

The imported rule constructs the literal HTML result and applies templates to
the source `greeting` element. Built-in element processing reaches its text
child. The principal text rule writes `!`, invokes `xsl:apply-imports`, and
then writes another `!`. Because no lower-precedence user text rule exists, the
apply-imports operation selects the built-in text rule and emits `Hello world.`

## Result

| Case | Expected | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0601` | HTML-shaped XML with `!Hello world.!` in `u` | XML-equivalent result | passed |

The executable test also observes the assembled matched-rule precedence vector
as exactly `[-1, 0]`. The conserved 16-case include denominator now has 6
explicit passes and 10 visible default not-run dispositions.

## Claim boundary

This evidence admits one imported simplified stylesheet whose implicit template
becomes a lower-precedence document rule, plus built-in text fallback from a
principal `text()` rule's `xsl:apply-imports`. It does not admit competing
document rules, imported modes, imported output declarations, multiple or
nested imports, include/import composition, or a general module graph.
