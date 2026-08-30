# XSLT30 `include-0103` Embedded Stylesheet Fragment and Base

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0103`
- Principal: `include-0103.xsl`
- Embedded resource: `include-0103a.xml#embedded`
- Nested module: `x/include-0103b.xsl`

## Execution

The resolver removes `#embedded` from the acquisition key and obtains only the
admitted `include-0103a.xml` bytes. After XML/XDM construction, XSLT-owned
fragment handling selects exactly one element whose `xml:id` is `embedded`.
The selected `xsl:stylesheet` remains within its original owned document, so
source locations, ancestors, namespace context, and base ancestry are not
recreated through serialization.

The selected element's `xml:base="x/"` resolves against the embedded resource
identity. Its child `xsl:include href="include-0103b.xsl"` consequently acquires
the admitted `x/include-0103b.xsl` resource. Compilation assembles the nested
module into the embedded module, then includes that program into the principal.
Named template `x` remains available to the principal root template.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0103` | `<a>found it</a>` | XML-equivalent result | passed |

The conserved 16-case denominator now has 8 explicit passes and 8 visible
not-run dispositions.

## Security and claim boundary

No path or URL is opened by the engine; all three resources were admitted into
one immutable snapshot. Fragment selection does not change the byte-acquisition
identity. Only a simple ASCII fragment selecting exactly one `xml:id` is
admitted. General XPointer, percent-decoded fragment names, DTD-typed IDs,
external entities, arbitrary nesting, and mixed include/import graphs remain
outside this result.

`include-0102` was evaluated as the initially smaller candidate but remains a
visible harness gap: its unnamespaced `id` is typed through an internal DTD, and
the accepted XML boundary forbids DTDs. FastXSLT did not preprocess the upstream
bytes or relax that security rule to manufacture a pass.
