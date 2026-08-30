# XSLT30 `include-0701` Two-Include Leaf-Import Graph

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0701`
- Principal: `include-0701.xsl`
- Included branches: `include-0701b.xsl`, `include-0701c.xsl`
- Imported leaves: `include-0701d.xsl`, `include-0701e.xsl`
- Source/result: file-backed `include-07.xml` and `include-0701.out`

## Assembly and execution

All source and stylesheet bytes are admitted into one immutable snapshot before
compilation. The loader prepares five module occurrences at maximum depth two.
Each included branch is compiled with its one leaf import, then both assembled
programs are included into the principal program.

Rules from the two included modules share principal import precedence `0`.
Their four imported-leaf rules retain lower precedence `-1`; six rules in the
assembled program have principal precedence. The later included `title` rule
wins the selected same-precedence conflict by declaration order. Its
`xsl:apply-imports` reaches the applicable imported `title` rule. Other rules
exercise built-in fallback and principal-over-import selection.

The newly admitted `/*` pattern is normalized as a document-element wildcard,
not as a stringly path approximation.

## Result

| Case | Expected | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0701` | File-backed XML assertion | XML-equivalent result | passed |

The conserved 16-case denominator now has 10 explicit passes and 6 visible
not-run dispositions.

## Claim boundary

This evidence admits one exact five-module topology: two includes from the
principal, each containing one leaf import. It also admits later-rule recovery
for the case's same-precedence match. It does not select configurable
`on-multiple-match`, the expected-error variant, distinct precedence ordering
between the two leaf-import branches, arbitrary graph recursion, repeated
module identity, or public dependency limits.
