# OASIS XSLT 1.0 Number Grouping Invalid Effective Value

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Does a statically known invalid effective value for the AVT-capable
`xsl:number` grouping attributes remain visibly invalid rather than appearing
to be an unsupported FastXSLT capability?

## Method

- Compile focused stylesheets with a multi-character grouping separator, zero
  grouping size, and nonnumeric grouping size.
- Require structured invalid diagnostic `XTDE0030`, the general error for an
  impermissible effective value of an AVT-capable attribute.
- Trace unchanged OASIS expected-error case
  `Microsoft/Number_GroupingSeperatorShouldBe1Char#1` through the conserved
  3,173-case runner.

## Result

The focused cases and unchanged OASIS case now report invalid `XTDE0030`. The
OASIS case already received initialization-time expected-error credit, so the
conserved totals remain 2,210 initialized cases, 2,157 successful executions,
and 2,009 / 3,173 (63.32%) exact expected-result matches.

The unsupported `FXST1051` frontier falls from two cases to one. The remaining
case uses genuinely dynamic grouping expressions and stays unsupported.

## Boundaries

- Valid static grouping behavior is unchanged.
- Detecting a statically known effective-value error during compilation does
  not redefine it as a stylesheet-grammar restriction.
- General grouping AVTs and their runtime validation are not admitted.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt number_rejects_invalid_static_grouping_values_without_calling_them_unsupported
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Number_GroupingSeperatorShouldBe1Char#1'
./scripts/verify.ps1
```
