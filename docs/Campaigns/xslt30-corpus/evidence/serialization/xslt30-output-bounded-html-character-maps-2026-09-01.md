# XSLT30 bounded HTML character maps -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0302`. It produces the
exact admitted serialization:

```html
<html><body><p>xx&xx</p><p>yy+Ayy</p><p>zz%&zz</p></body></html>
```

This satisfies the pinned optional-DOCTYPE serialization pattern without an
XML declaration. Three character maps replace `#` with raw `&`, `*` with `+`,
and `$` with `A`.

## Boundary conservation

This is not general HTML serialization. Compilation admits `method="html"`
only when a character map is selected. Runtime execution additionally requires
one null-namespace `html` result containing only the admitted `body` and `p`
element shape, without result attributes, namespace nodes, or processing
instructions. Any wider result remains explicitly unsupported.

A focused negative compiler test proves that ordinary HTML output without this
bounded character-map profile still reports `FXST1004`. HTML void elements,
raw-text elements, URI escaping, metadata injection, doctypes, case folding,
boolean attributes, and general HTML character escaping remain unclaimed.

## Denominator movement

The complete output denominator moves from 104 to 105 passes and from 128 to
127 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 302 to 303 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 175 visible default not-run cases.
