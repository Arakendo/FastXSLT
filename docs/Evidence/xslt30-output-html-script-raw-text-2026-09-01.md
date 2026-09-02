# XSLT30 HTML script raw text -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0154` case. The stylesheet
manually escapes `<EM>` source characters so they become ordinary characters in
the semantic script text. Under the HTML method, the serializer emits that text
without XML escaping, producing the expected JavaScript string.

## Admission boundary

The private validator admits only an attribute-free no-namespace `html` with
one `head` and empty `body`; the head contains one `script` with exactly
`type="text/javascript"` and one text child. This does not admit arbitrary HTML
trees or the broader `script`, `style`, `pre`, and `textarea` whitespace case.
XHTML continues to use XML-compatible escaping for the same element names.

## Denominator movement

The complete output denominator moves from 175 to 176 passes and from 56 to 55
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 373 to 374 passes
and from 104 to 103 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
