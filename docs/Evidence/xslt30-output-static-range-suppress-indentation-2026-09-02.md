# XSLT30 static range and inherited suppress indentation -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0232` case. Both
`xsl:for-each select="1 to 5"` instructions emit five result elements, and the
serializer satisfies all seven native `serialization-matches` assertions for
indentation outside `chapter` plus adjacency inside the unprefixed `p` and
namespace-qualified `z:p` subtrees.

The case exposed and fixed an inherited-state defect in the earlier
`suppress-indentation` slice. Suppression attached to a matching element now
remains active for its complete descendant subtree; indentation no longer
restarts inside nested `wizard` or `hobbit` elements.

## Admitted execution boundary

The compiler admits an exact static signed-integer range only when its body is
recursively limited to literal result elements and text. Each range item is
charged before its body executes, a descending range is empty, and an
unrepresentable host-size range fails explicitly. A body that needs the atomic
context item remains unsupported because the current private sequence context
does not represent atomic focus; FastXSLT does not substitute the source node
or fabricate a value.

This is a safe bounded execution slice, not general XPath range evaluation or
general atomic-focus support.

## Denominator movement

The complete `decl/output` denominator moves from 191 to 192 passes and from 40
to 39 visible default not-run cases; its one profile exclusion is unchanged.
Across the eleven conserved XSLT30 denominators, the total moves from 389 to
390 passes and from 88 to 87 visible default not-run cases, with three
engine-unsupported cases and 51 profile exclusions unchanged.
