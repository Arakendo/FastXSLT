# XSLT30 `conflict-resolution-0702` Literal Static Context

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0702`
- Stylesheet: `conflict-resolution-0702.xsl`
- Environment: shared `conflict-resolution-07` principal source

## Method

The metadata-driven helper executes the pinned source and stylesheet through a
bounded sealed snapshot and identified batch of one. The compiler admits
`xsl:xpath-default-namespace` as the sole XSLT-namespaced static-context
attribute on a literal result element. It influences descendant compilation but
is not lowered as a result attribute.

The descendant simple `foo` selection lowers to an expanded-name child
selection using `http://some.uri/`. The template's retained `u` namespace
binding remains available to literal-result serialization. The test compares
the result element name and string value, verifies the `u` declaration, and
proves the static-context attribute is absent from serialized output; an XML
declaration is allowed because the stylesheet explicitly selects XML output.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0702` | `Match-of-qualified-name` | equal | passed |

The XSLT-namespaced attribute supplies static context only. The result is an
unnamespaced `out` element retaining `xmlns:u="http://some.uri/"` and no
`xpath-default-namespace` result attribute.

## Claim boundary

This evidence admits only `xsl:xpath-default-namespace` as a control attribute
on literal result elements. Narrow unnamespaced literal result attributes and
whole-value variable attribute value templates are evidenced separately by
`conflict-resolution-1205`. Other XSLT control attributes, general attribute
value templates, namespace aliases, and broader
default-namespaced paths remain outside this slice. Stylesheet-wide inheritance
is evidenced separately by `conflict-resolution-0703`.
