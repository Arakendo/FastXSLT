# QT3 deep-equal untyped/duration boundary -- 2026-09-02

## Result

FastXSLT now executes unchanged QT3 `cbcl-deep-equal-008`. The private atomic
model retains `xs:untypedAtomic` independently from strings and adds a bounded
`xs:yearMonthDuration` representation normalized to checked signed months.
Literal decimals are retained as exact decimals rather than binary floats.

The first pair of untyped `"a"` values compares equal. The second pair remains
unequal because untyped atomic `"P1Y"` and typed year-month duration `P12M` do
not acquire the same atomic type merely because the lexical values could be
cast to equivalent durations. Evaluation therefore returns the native
`assert-false` result after the length decision and two reached items, charging
exactly three XPath operations.

## Conservation

The immutable case identity is an explicit `selected/passed` ledger record.
Deep-equal advances from 183 to 184 passes and its visible defaults fall from
13 to 12. The combined 612-case subtotal remains conserved as 408 passes, 179
profile exclusions, and 25 visible defaults.

## Boundary

The duration parser covers the year/month lexical shape required by this case
with checked arithmetic. This slice does not admit general duration arithmetic,
day-time durations, implicit casting of untyped atomic values, schema-typed
nodes, UCA/private collations, or invocation clock/timezone semantics.
