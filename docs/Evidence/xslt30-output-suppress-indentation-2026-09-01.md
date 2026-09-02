# XSLT30 suppress indentation -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0725` and `output-0726`
cases. Both compile `suppress-indentation="p"` to the unnamespaced expanded name
and preserve the complete long paragraph without inserting whitespace or line
breaks inside it under HTML 5 and XHTML serialization.

A focused compiler sentinel also compiles `p z:p` and verifies that the second
name retains `http://example.com/z` rather than merging by lexical local name.
Multiple declarations union non-duplicate expanded names through the existing
bounded output-declaration merge.

## Boundary

The serializer consults suppression only when deciding whether to insert its
own child indentation. It does not remove semantic whitespace, suppress content,
or implement word wrapping. The HTML admission guard remains the exact
attribute-free `html/head/title/body/h1/p` hierarchy used by the corpus case;
XHTML remains on the XML-compatible serializer path.

## Denominator movement

The complete output denominator moves from 178 to 180 passes and from 53 to 51
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 376 to 378 passes
and from 101 to 99 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
