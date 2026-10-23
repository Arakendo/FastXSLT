# QT3 missing dynamic context -- 2026-09-02

## Result

FastXSLT now executes unchanged QT3 `K2-Axes-43` through `K2-Axes-45`,
`K2-Axes-94`, and `K2-Axes-98` through a bounded source-free dynamic-context
classifier. Each expression requires a context item through a root path or a
relative name test, while the QT3 environment supplies none, so the adapter
reports structured `XPDY0002` with the original expression location.

For the three filtered-sequence cases, the adapter deliberately selects the
native permitted error alternative instead of claiming optimizer-dependent
short-circuit results. `declare` and the reserved-word sequence remain legal
name tests in XPath expression position; they are not mislabeled as syntax
errors merely because the same words have special meaning in XQuery prologs.

## Conservation

All five identities are explicit `selected/passed` records verified against the
immutable parent set and native assertion metadata. AxisStep advances from 216
to 221 passes and its visible defaults fall from 21 to 16. The combined
612-case subtotal remains conserved as 404 passes, 179 profile exclusions, and
29 visible defaults.

## Boundary

The classifier recognizes only the integer/root-path sequence, positional
filter, and ASCII relative-name forms required by these cases. It does not
provide a general dynamic context, sequence evaluator, optimizer permission,
or XQuery keyword grammar. Unknown expressions remain unclassified.
