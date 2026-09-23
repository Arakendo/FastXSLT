# OASIS XSLT 1.0 Output-Declaration Recovery

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can FastXSLT admit the XSLT 1.0 recovery choice for conflicting output
properties without weakening modern-version validation or guessing across lost
import-precedence provenance?

## Method

- Preserve strict repeated-property validation for XSLT 2.0/3.0 stylesheets.
- For declarations compiled directly within an XSLT 1.0 stylesheet, recover a
  conflicting scalar property by choosing the value from the later declaration.
- Continue accumulating additive `cdata-section-elements` names.
- Treat an explicitly repeated `omit-xml-declaration="no"` as a real later
  value rather than combining booleans with logical OR.
- Keep cross-module merging strict because an already compiled included module
  may contain output properties inherited from a lower-precedence import.

## Result

Six direct repeated-output cases leave `FXST1018` and reach execution or source
admission. `Lotus/output_output87#1`, whose catalog explicitly selects the
`choose-last` discretionary behavior, compares exactly.

The remaining cases expose later independent boundaries:

- one source document is invalid under the XML adapter;
- one text-output case reaches an output mismatch;
- one reaches an XML serialization cardinality error;
- two reach output/comparator behavior that differs from their archival
  expectations.

`Microsoft/Output_Output_PreserveImportPrecedenceOnOuputElement#1` remains
`FXST1018`: its lower-precedence imported `indent` setting has been flattened
into an intermediate included program, so treating it as a same-precedence
later declaration would select the wrong value.

The complete sweep now initializes 2,186 cases, reports 949 initialization
failures, executes 2,129 successfully, and reports 57 execution failures.
Exact XML-semantic matches rise from 1,979 to 1,980. The strict compatibility
lower bound is `1,980 / 3,173 = 62.40%`.

## Boundaries

- Recovery is explicitly XSLT 1.0-only; modern conflicting declarations remain
  rejected by the private bounded merger.
- This does not select a general recovery policy for unrelated XSLT errors.
- Correct nested import/include output precedence requires property-level
  provenance, not a last-value guess.
- Execution, serialization, comparator, and archival-expectation differences
  remain visible after initialization succeeds.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features rejects_overlapping_output_properties_during_bounded_merge
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1018
./scripts/measure-oasis-xslt10.ps1 -TraceCase output87
./scripts/verify.ps1
```
