# XSLT30 HTML normalization none -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0161` case. The compiled
HTML output settings retain `normalization-form="none"`, and serialization
preserves the authored decomposed `A` plus combining acute accent as the exact
UTF-8 byte sequence `41 CC 81`.

This composes the HTML method with the same non-normalizing behavior already
exercised by XML, XHTML, and text cases. It does not infer Unicode NFC support.
In particular, `output-0164` remains outside the admitted slice because its
default URI-escaping result first requires normalization of a decomposed
character before percent encoding.

## Denominator movement

The complete output denominator moves from 171 to 172 passes and from 60 to 59
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 369 to 370 passes
and from 108 to 107 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
