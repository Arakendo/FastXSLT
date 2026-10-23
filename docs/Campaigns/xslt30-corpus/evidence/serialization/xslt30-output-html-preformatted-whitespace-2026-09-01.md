# XSLT30 HTML preformatted whitespace -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0159` case. Under HTML
serialization with `indent="no"`, script and style text is emitted without XML
escaping, and authored newlines, tabs, and spaces remain intact inside the
script, style, pre/b, and textarea content.

The executable sentinel covers every active W3C assertion family in the case:
the Content-Type meta inserted into `head`, raw script content, raw style
content, and the significant textarea text. It additionally checks the bold
text inside `pre`, whose upstream assertion is currently commented out.

## Admission boundary

The private validator admits only the exact no-namespace hierarchy used by this
case: `html` contains `head` and `body`; head contains the previously admitted
typed script followed by one text-only style; body contains one pre with a
single text-only bold child followed by one text-only textarea whose attributes
are exactly `rows="2"` and `cols="20"`. General HTML trees and indentation
semantics remain unsupported.

## Denominator movement

The complete output denominator moves from 176 to 177 passes and from 55 to 54
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 374 to 375 passes
and from 103 to 102 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
