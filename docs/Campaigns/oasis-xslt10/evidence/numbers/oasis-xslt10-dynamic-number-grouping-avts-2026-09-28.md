# OASIS XSLT 1.0 Dynamic Number Grouping AVTs

Date: 2026-09-28  
Status: Local compatibility evidence

## Question

Can the compatibility path evaluate `xsl:number/@grouping-separator` and
`@grouping-size` attribute value templates without weakening the XSLT 1.0
one-character separator rule or creating a second number formatter?

## Method

- Move grouping from the statically parsed number-format value into a typed
  instruction-owned plan.
- Reuse the charged XSLT 1.0 concat/variable evaluator for the separator and
  the charged binary-numeric evaluator for the size.
- Resolve the effective grouping values per invocation and require exactly one
  separator character plus a positive integral size.
- Compose the grouping plan with both static and dynamic number-format plans.
- Trace unchanged OASIS cases `Microsoft/BVTs_bvt061` and
  `Microsoft/Number__91026` through the conserved 3,173-case runner.

The one-character rule comes from the XSLT 1.0 `xsl:number` syntax, which types
`grouping-separator` as an AVT yielding `char`:
[XSLT 1.0 section 7.7](https://www.w3.org/TR/xslt-10/#number).

## Result

The focused reference case evaluates `grouping-size="{$grouping-size + 1}"`
and `grouping-separator="{concat($grouping-separator, '')}"`, composes them
with a dynamic `format` AVT, and produces `012,345`.

Both unchanged OASIS stylesheets advance past grouping compilation to their
independent dynamic `letter-value` boundary. Neither earns pass credit:

- `BVTs_bvt061` later requires unsupported traditional Hebrew and Chinese
  numbering behavior.
- `Number__91026` also evaluates its first grouping separator to the
  two-character string `,.`; FastXSLT deliberately retains the normative
  one-character rule instead of adopting that historical expected output.

The conserved totals therefore remain 2,229 / 3,173 exact matches (70.25%),
2,440 initialized cases, and 2,392 successful executions. The former
`FXST1051` dynamic-grouping frontier is eliminated; the next visible boundary
is correctly attributed to dynamic language/letter-value semantics.

## Boundaries

- This admits only whole-expression AVTs represented by the existing typed
  variable, concat, and binary-numeric plans.
- Effective values are checked at invocation time; malformed or nonconforming
  values remain structured `XTDE0030` failures.
- Modern grouping behavior is unchanged.
- Traditional/language-sensitive numbering sequences are not inferred from
  decimal fallback behavior.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt xslt10_number_resolves_dynamic_grouping_avts
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt061'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Number__91026'
./scripts/verify.ps1
```
