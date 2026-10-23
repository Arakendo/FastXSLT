# XSLT30 `include-0202` Imported Parameter and Computed Attribute

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0202`
- Principal stylesheet: `include-0202.xsl`
- Imported stylesheet: `include-0202a.xsl`
- Source: inline `<doc>This text should be output</doc>`

## Execution

The source and both stylesheet modules are copied into a bounded sealed resource
snapshot. The principal `doc` rule calls `xsl:apply-imports` with integer
parameter `magic=81`. Selection reaches the lower-import-precedence `doc` rule,
which binds that invocation-local parameter.

The imported rule's leading `<xsl:attribute name="magic">` is compiled into a
private computed-attribute plan owned by its literal `in` result element. The
exact admitted value constructor is one child `xsl:value-of` selecting the
bound `$magic` variable. Runtime materializes the attribute before result
children and charges it as one result node.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0202` | `<out><in magic="81"/></out>` | XML-equivalent result | passed |

The conserved 16-case include denominator now has 4 explicit passes and 12
visible default not-run dispositions.

## Claim boundary

This evidence admits one integer parameter on `xsl:apply-imports`, binding by an
imported matched template, and one leading computed attribute with a static
unnamespaced NCName and variable value on a literal result element. It does not
admit computed or namespaced attribute names, literal/mixed/general attribute
sequence constructors, attributes after child construction, standalone
computed-attribute execution, multiple imports, or broader imported
declarations.

The specialization remains private. Semantic inspection counts the computed
attribute independently even though execution stores it with the owning result
element plan.
