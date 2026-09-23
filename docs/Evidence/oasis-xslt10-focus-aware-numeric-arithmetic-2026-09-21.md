# OASIS XSLT 1.0 Focus-Aware Numeric Arithmetic

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can the shared checked numeric-expression tree evaluate `position()` and
`last()` as true dynamic-focus operands, including beside variables, without a
one-off parser for a corpus expression or an invented default focus?

## Method

- Add typed context-position and context-size leaves to the existing
  exact-rational numeric plan.
- Pass the active sequence focus explicitly from value-of, template argument,
  variable, copy-of, and computed-attribute consumers.
- Charge each focus read as XPath work and report `XPDY0002` when the plan is
  evaluated without a dynamic focus.
- Add exact `floor()` composition and let `xsl:number/@value` reuse the same
  compiled plan, active focus, runtime variable frame, diagnostics, and work
  accounting.
- Add exact XPath `round()` composition to the same plan, including the
  negative-half rule that rounds toward positive infinity.
- Add a complete apply-templates regression for
  `$offset+position()+last()` over three selected nodes.
- Add a complete number regression for
  `(floor(position() div 12)*50)+(position() mod 12)-1`.
- Trace unchanged Microsoft `Variables__84636` and Lotus
  `numbering_numbering17` and `math_math101`, then rerun the conserved OASIS
  measurement.

## Result

The focused lifecycle produces `14`, `15`, and `16`, proving that both position
and size come from the apply-templates selection rather than a fabricated
singleton focus. Unchanged Microsoft `Variables__84636` clears its former
invalid variable-expression frontier and produces the expected transform
content, including positions `2`, `4`, and `6`. It remains a visible comparison
mismatch because the expected HTML serialization is not an XML document or
fragment equivalent to FastXSLT's normalized HTML result.

The number regression proves exact floor/divide/modulo/add/subtract composition
across focus positions and the 11-to-12 boundary. Unchanged Lotus
`numbering_numbering17` clears `FXXP1022` and becomes exact.

The arithmetic regression also preserves operator precedence across path,
literal, variable, modulo, and `round()` operands, including
`round(-2.5) = -2`. Unchanged Lotus `math_math101` now emits `15,2,2` exactly.

Four additional catalog cases now initialize and execute. The conserved totals
are 2,134 initialized cases, 2,045 successful executions, 89 execution
failures, 1,912 exact XML-semantic matches, and 55 mismatches. The strict
compatibility lower bound is now `1,912 / 3,173 = 60.26%`.

## Boundaries

- The change adds typed numeric operands plus exact unary floor and round, not
  a general XPath function-call evaluator.
- Focus is supplied explicitly per execution site and is not retained in the
  compiled program or prepared input.
- Focusless evaluation fails rather than assuming position and size are one.
- HTML result comparison remains a separate harness concern; the corpus case
  receives no pass credit from visually equivalent content.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_binary_numeric_tree_uses_sequence_position_and_size
cargo test -p fastxslt --all-features xslt10_number_reuses_focus_aware_checked_arithmetic
cargo test -p fastxslt --all-features xslt10_binary_numeric_tree_composes_round_and_modulo
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/Variables__84636
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/numbering_numbering17
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/math_math101
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
