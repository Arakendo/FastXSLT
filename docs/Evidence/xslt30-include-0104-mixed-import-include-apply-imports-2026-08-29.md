# XSLT30 `include-0104` Mixed Import/Include Apply-Imports

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0104`
- Principal: `include-0104.xsl`
- Imported module: `xinc20a.xsl`
- Included module: `xinc20b.xsl`
- Source: shared inline `include-01` environment

## Assembly and execution

The sealed dependency graph contains exactly three modules. The principal's
`xsl:import` is first among its top-level declarations and contributes a
`one-tag` rule at import precedence `-1`. The following `xsl:include` contributes
another `one-tag` rule at the principal module's precedence `0`. The principal
`root-tag` rule also remains at precedence `0`.

Compilation observes the assembled matched-rule precedence vector as exactly
`[-1, 0, 0]`. Initial processing selects the principal `root-tag` rule, which
applies templates to `one-tag`. Normal conflict resolution selects the included
rule at precedence `0`. Its `xsl:apply-imports` then considers only lower
precedence and selects the imported rule at `-1`.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0104` | `<from-xinc20b><from-xinc20a/></from-xinc20b>` | XML-equivalent result | passed |

The conserved 16-case denominator now has 9 explicit passes and 7 visible
not-run dispositions.

## Claim boundary

This evidence admits one principal module with exactly one leading import and
one following include, each a leaf dependency. It proves that inclusion does
not manufacture import precedence and that apply-imports crosses from the
included rule to the imported rule. It does not admit arbitrary declaration
order, multiple includes in one module, nested imports, mixed nested graphs,
output-declaration merging, or general precedence-graph construction.
