# OASIS XSLT 1.0 Dynamic Number-Format Parameter

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can an XSLT 1.0 `xsl:number/@format` attribute resolve a named-template
parameter at invocation time without specializing call sites, mutating the
compiled program, or widening modern numbering semantics?

## Method

- Retain static number formats as pre-parsed immutable plans.
- Compile the exact XSLT 1.0 variable AVT shape `{$name}` as a distinct typed
  format plan carrying only the variable name.
- At each invocation, reuse the existing charged XSLT 1.0 variable string
  conversion and parse the resulting lexical format through the same admitted
  pure parser used for static formats.
- Exercise two calls to one named template with different format parameter
  values in a focused runtime test.
- Run unchanged OASIS case `Lotus/namedtemplate_namedtemplate12#1` through the
  complete 3,173-case measurement harness.

## Result

The focused test resolves `A. ` and `(001)` independently through one compiled
named-template body. The unchanged OASIS case resolves `A. ` and `a. ` across
nested list calls and matches its admitted expected XML result.

The conserved sweep moves from 2,209 to 2,210 initialized cases, from 2,156 to
2,157 successful executions, and from 2,008 to 2,009 exact expected-result
matches. The rounded percentage remains 63.32% of the 3,173-case catalog. No
new mismatch or execution failure is introduced, and the unsupported
`FXST1049` initialization frontier falls from four cases to three.

## Boundaries

- The new dynamic plan is selected only for XSLT 1.0 compatibility semantics;
  modern profiles retain their existing static-format boundary.
- Only one variable reference occupying the complete AVT is admitted. General
  AVT concatenation, path expressions, functions, and dynamic
  `letter-value`/grouping composition remain unsupported.
- Dynamic values are limited to the already admitted decimal, Latin
  alphabetic, Roman, prefix, suffix, and separator grammar. An unimplemented
  runtime token reports structured unsupported diagnostic `FXRT1017`.
- The compiled program retains no invocation value or mutable cache.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt xslt10_number_resolves_a_named_template_format_parameter_per_call
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/namedtemplate_namedtemplate12#1'
./scripts/verify.ps1
```
