# QT3 Axis Static Syntax Errors -- 2026-09-02

## Result

Seven unchanged cases from pinned QT3 `prod/AxisStep.xml` now execute through
the private location-path parser and its owned static-diagnostic assertion:

- `Axes088`;
- `K2-Axes-34`, `K2-Axes-35`, and `K2-Axes-46`;
- `K2-Axes-77`; and
- `K2-Axes-90` and `K2-Axes-91`.

The cases cover a trailing empty step, a bare descendant separator, an unknown
axis name, and an incomplete QName. Each is classified as invalid syntax with
the standard `XPST0003` code and retains its expression source location.
Where QT3 permits more than one outcome, FastXSLT takes the explicit
`XPST0003` alternative.

## Implementation boundary

The location-path failure now carries its standard static-error code instead of
being mapped to a private invalid-expression code. Forms that are syntactically
valid but outside the deliberately narrow parser remain `Unsupported`; this
slice does not turn unsupported XPath, XQuery declarations, namespace-axis
semantics, or static typing into syntax errors.

The adapter reads the native expression and error assertion from the immutable
test set, verifies the exact expected expression, and requires an
`Invalid/XPST0003` result. No source document is manufactured because static
parsing must fail before dynamic context is relevant.

## Accounting

The AxisStep denominator advances from 182 to 189 passes, retains 112
XQuery-profile exclusions, and reduces visible default not-run cases from 55
to 48. Across the two active QT3 denominators, the conserved subtotal is now
340 passes, 179 profile exclusions, and 93 visible default not-run cases out of
612.

