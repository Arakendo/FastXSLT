# XSLT30 HTML 5 element namespace normalization -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0602a`, `output-0602b`, and
`output-0602c` cases. The output harness resolves the environment-supplied
stylesheet and file-backed source when the case-local `<test>` is empty, then
imports both resources into the bounded sealed snapshot.

HTML 5 serialization normalizes prefixed and unprefixed SVG and MathML elements
to their respective default namespaces. Known SVG/MathML prefix declarations
are removed from ancestor HTML elements. The unrelated `NamespaceN` binding and
`n:zzz` element remain qualified and intact.

## Boundary conservation

The admitted result vocabulary is fixed to the HTML elements already required
by `output-0601`, SVG `svg`/`rect`/`circle`, MathML
`math`/`mrow`/`mi`/`msup`/`mn`/`mo`, and the exact foreign `n:zzz` control.
Only the source's unnamespaced, non-URI geometry/presentation attributes are
accepted. URI attributes, arbitrary foreign namespaces, namespaced attributes,
raw-text behavior, and general HTML namespace fixup remain unsupported.

## Denominator movement

The complete output denominator moves from 142 to 145 passes and from 90 to 87
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 340 to 343 passes, with 3 engine unsupported cases, 50
profile exclusions, and 135 visible default not-run cases.
