# XSLT30 HTML version control characters -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0195b` case. Its explicit
HTML serialization `version="5.0"` causes the `#x9F` result-tree character to
serialize as the hexadecimal numeric reference `&#x9F;`, satisfying the
upstream assertion without placing the control character directly in output.

The sibling `output-0195` declares an HTML 4 dependency and is now explicitly
excluded by the selected FastXSLT output profile. `output-0195a` remains a
visible default not-run case because its stylesheet omits the version and its
test environment supplies the HTML 5 default; this slice does not silently turn
suite metadata into an engine default.

## Boundary conservation

Admission is limited to explicit HTML version 5, one unnamespaced `doc` result
element with no attributes or namespace bindings, and a text child containing a
C1 control. This does not admit HTML 4, a configurable default HTML version, or
an arbitrary HTML element vocabulary.

## Denominator movement

The complete output denominator moves from 149 passes and 83 visible default
not-run cases to 150 passes, 1 profile exclusion, and 81 visible default not-run
cases. Across the eleven conserved XSLT30 denominators, the total moves from 347
to 348 passes, from 50 to 51 profile exclusions, and from 131 to 129 visible
default not-run cases; 3 engine-unsupported cases remain unchanged.
