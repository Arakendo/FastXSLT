# OASIS XSLT 1.0 Dynamic Number Control AVTs

Date: 2026-09-29  
Status: Local compatibility evidence

## Question

Can the XSLT 1.0 compatibility lane evaluate bounded `xsl:number/@lang` and
`@letter-value` attribute value templates, plus constant `format` AVTs,
without introducing locale authority or weakening the normative one-character
`grouping-separator` rule?

## Method

- Compile a whole-expression XSLT 1.0 control AVT into a private typed plan
  when it is either one unqualified variable reference or an already admitted
  `concat()` expression.
- Evaluate that plan through the charged invocation-owned value evaluator.
- Validate the effective `letter-value` as `alphabetic` or `traditional` and
  retain the existing implemented-token admission rules.
- Validate the effective language only after resolving the effective number
  format. Decimal fallback remains language-invariant; no ambient locale is
  consulted.
- Constant-fold quoted `format` AVTs before parsing the number-format token.
- Differentially trace unchanged OASIS cases `Microsoft/BVTs_bvt061#1` and
  `Microsoft/Number__91026#1` through the conserved 3,173-case runner.

The invocation-time validation is owned by a private submodule rather than
adding another responsibility to the already large number executor.

## Result

The focused reference case resolves:

- `format="{$format}"` with
  `letter-value="{concat($letter-val, 'itional')}"`;
- the empty constant AVT `format="{''}"`; and
- `format="{'丁'}"` with `lang="{concat($lang, 'o')}"`.

It produces the bounded text result `3999|99999|1000`. The dynamic control
values are invocation state; the compiled stylesheet retains only immutable
typed expression plans.

The unchanged OASIS cases do not earn exact-pass credit:

- `Microsoft/Number__91026#1` now initializes, then correctly reports
  `XTDE0030` because its first AVT evaluates `grouping-separator` to the
  two-character string `,.`. XSLT 1.0 declares this attribute as `char`.
- `Microsoft/BVTs_bvt061#1` still reports `XTDE0030` during initialization for
  a later static `grouping-size="0"` instruction in the larger stylesheet.

The strict lower bound therefore remains 2,262 / 3,173 exact matches (71.29%).
Initialized cases increase from 2,442 to 2,443; initialization failures fall
from 728 to 727; successful executions remain 2,393; and execution failures
increase from 49 to 50. Visible mismatches and comparator gaps remain zero.
This is useful frontier movement, not a manufactured pass over a nonconforming
expected result.

## Boundaries

- This admits only whole-expression variable and `concat()` AVTs for these two
  controls. It does not expose a general AVT type.
- Invalid effective `letter-value` values report structured `XTDE0030`.
- Locale-sensitive numbering sequences remain unsupported unless already
  implemented and explicitly admitted.
- The one-character grouping separator contract is unchanged.
- Modern numbering behavior is unchanged.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt xslt10_number_resolves_dynamic_letter_value_language_and_literal_format_avts
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt061#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Number__91026#1'
./scripts/verify.ps1
```
