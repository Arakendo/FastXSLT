# OASIS XSLT 1.0 Included Output Import Precedence

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can an included stylesheet retain enough output-property provenance for the
principal module to distinguish its same-precedence declarations from output
properties inherited through an import?

## Method

- Retain a private set of output properties declared at the compiled module's
  current precedence. The existing effective-property set continues to include
  both current and inherited properties.
- Keep imported properties out of that current-precedence set while carrying
  current-precedence properties through `xsl:include` composition.
- Before the ordinary output merge, remove a shadowed inherited `method`,
  `encoding`, or `indent` value when the competing property is known to be at
  the including module's higher precedence.
- Preserve the existing same-precedence merge and conflict checks when both
  properties originate at the current precedence or when neither side has
  enough provenance to establish a winner.
- Add focused sealed-resource graphs for both directions: a principal property
  shadowing an import reached through an include, and an included property
  shadowing an earlier principal import.

## Result

The unchanged
`Microsoft/Output_Output_PreserveImportPrecedenceOnOuputElement#1` case now
initializes, executes, and compares exactly. The final `FXST1018` observation
is eliminated.

The complete 3,173-case sweep initializes 2,195 cases, reports 940
initialization failures, executes 2,138 successfully, and reports 57 execution
failures. Exact XML-semantic matches rise from 1,986 to 1,987; mismatches remain
67 and comparator-unsupported results remain 75. The strict compatibility
lower bound is `1,987 / 3,173 = 62.62%`.

## Boundaries

- This is private property-level precedence provenance, not a public compiled
  stylesheet representation.
- The admitted inherited scalar properties remain `method`, `encoding`, and
  `indent`; arbitrary lower-precedence output composition is not inferred.
- Equal-precedence conflicts still use the existing XSLT 1.0 recovery policy
  and strict modern-version behavior.
- The change does not flatten module graphs, introduce resource access, or
  alter template/import precedence.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features output_shadows
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/Output_Output_PreserveImportPrecedenceOnOuputElement#1
./scripts/verify.ps1
```
