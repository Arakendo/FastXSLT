# QT3 deep-equal literal composite updates -- 2026-09-02

## Result

FastXSLT now executes unchanged QT3 `fn-deep-equal-arrays-18` and
`fn-deep-equal-maps-15` from pinned `fn/deep-equal.xml`. The safe private
composite owner folds these bounded literal operations before comparison:

- `array:put` replaces one one-based member sequence;
- `array:remove` removes one one-based member; and
- `map:remove` removes one admitted numeric key.

The original parsed values remain immutable. The resulting values use the same
recursive equality and exact XPath-operation accounting as the preceding array
and map tranches. The adapter validates unchanged identity, expression,
assertion, result, operation count, and zero node visits.

## Conservation and boundary

The 263-case deep-equal denominator now records 183 passes, 67 profile
exclusions, and 13 visible defaults. The combined 612-case QT3 subtotal is 387
passes, 179 exclusions, and 46 visible defaults.

Only literal operations over the already admitted composite subset are folded.
Invalid or zero positions, missing removal keys, dynamic operands, general
function invocation, mutation, and public array/map update APIs remain outside
this slice. This is a bounded semantic reference, not an optimizer or a public
representation decision.
