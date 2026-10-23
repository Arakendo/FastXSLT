# XSLT30 call-template for-each argument constructor

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 case `call-template-1301` passes through the normal
sealed-resource, compilation, transform-set, and XML-comparison path.

The case constructs one named-template argument with:

```xml
<xsl:with-param name="pvar1">
  <xsl:for-each select="a">
    <xsl:value-of select="."/>
  </xsl:for-each>
</xsl:with-param>
```

FastXSLT already owned this bounded path-selection/string-value constructor for
XSLT 1.0 compatibility work. The compiler and private plan now describe it as a
version-neutral constructor because the same semantics are valid for this XSLT
2.0 case. The argument remains an invocation-owned temporary tree and is passed
without changing the caller's source focus.

## Boundary

This admission remains deliberately narrow: one `xsl:for-each`, a supported
location path, and exactly one `xsl:value-of select="."` child. It does not admit
arbitrary sequence constructors, nested control flow, general XPath iteration,
or a second execution backend.

## Accounting

The complete 42-case `insn/call-template` denominator advances from 25 to 26
passes, with one profile exclusion and 15 visible defaults. Across the conserved
XSLT30 denominators this produces 497 passes, 12 engine-unsupported cases,
2,879 profile exclusions, and 111 visible defaults out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt30_call_template_inventory_tests --all-features`
