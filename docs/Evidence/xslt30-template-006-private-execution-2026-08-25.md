# XSLT30 `template-006` Private Execution

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/template/_template-test-set.xml` |
| Case | `template-006` |
| Upstream dependency | `spec = XSLT20+` |
| Local selection | `corpus/overlays/xslt30/private-slice-v0.toml` |
| Claim | One private case executed; no conformance claim |

## Why this case

The case requires one root template and one empty unnamespaced literal result
element. It fits the existing private semantic slice while adding useful
pressure absent from the hand-authored golden: the stylesheet has no
`xsl:output` declaration. FastXSLT must preserve that absence and infer the XML
method for this non-HTML result instead of requiring or inventing an explicit
declaration during compilation.

## Harness path

The test locates `template-006` in the unmodified upstream test-set document,
resolves its named environment, reads the principal source content, resolves
the referenced stylesheet file, and reads the inline `assert-xml`. Source and
stylesheet bytes are then admitted under logical identities to a sealed
in-memory resource snapshot before compilation and execution.

The import reads close before the transform begins. The engine never receives
the suite path and performs no ambient filesystem access during compilation or
execution. The overlay stores only selection and rationale; it does not copy
the upstream fixture or expected result into FastXSLT-owned corpus.

## Result

FastXSLT serializes an XML declaration followed by an expanded empty element.
The upstream assertion uses the self-closing form `<o/>`. Both are parsed and
compared as the same empty document element, consistent with an XML assertion
rather than a byte-exact serialization assertion.

The focused case and all repository gates pass as part of 27 unit tests. This
proves the first suite-linked harness path and one common semantic behavior. It
does not establish XSLT 2.0 or 3.0 support, a pass-rate denominator, broad
dependency handling, or a public transformation API.
