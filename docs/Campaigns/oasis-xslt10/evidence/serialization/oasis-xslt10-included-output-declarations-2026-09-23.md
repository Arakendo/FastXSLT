# OASIS XSLT 1.0 Included Output Declarations

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can output declarations contributed through `xsl:include` participate in the
same-precedence principal output definition without losing property conflict
checks or import-precedence boundaries?

## Method

- Reconstruct the private output declaration metadata retained by each compiled
  module: settings, explicitly specified properties, character-map names, and a
  diagnostic location.
- Merge included and principal declarations through the existing
  same-precedence `merge_output` oracle rather than copying a completed output
  object wholesale.
- Admit a repeated `omit-xml-declaration` property only when both declarations
  specify the same value, matching the existing compatibility rule for method,
  encoding, and indentation.
- Preserve character-map finalization until after module composition.
- Retain unsupported classification for conflicting same-precedence properties
  and for lower-precedence output metadata whose provenance has been flattened
  by an intermediate compiled module.

## Result

The former seven-case `unsupported/FXST1019` frontier is eliminated. The
unchanged `Lotus/impincl_impincl01#1` case now initializes, executes, and
compares exactly.

The other six cases progress to the next honest boundary:

- three import/include cases reach duplicate included-template selection
  (`FXST1021`);
- two include cases reach duplicate root-template selection (`FXST1020`);
- one nested import/include output case reaches `FXST1018` because the
  intermediate compiled module no longer distinguishes its imported
  lower-precedence `indent` property from same-precedence included properties.

The complete sweep now initializes 2,171 cases, reports 964 initialization
failures, executes 2,115 successfully, and reports 56 execution failures.
Exact XML-semantic matches rise from 1,968 to 1,969, with 65 mismatches. The
strict compatibility lower bound is `1,969 / 3,173 = 62.05%`.

## Boundaries

- This does not flatten `xsl:include` textually or erase import precedence.
- Conflicting values at equal precedence remain outside the admitted recovery
  policy.
- Imported output-property provenance through a subsequently included module
  was unresolved in this tranche; it is now addressed by the later
  [property-provenance evidence](oasis-xslt10-included-output-import-precedence-2026-09-23.md).
- No output setting or module-composition type becomes public.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features included_output_declarations_merge_at_principal_precedence
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1019
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/impincl_impincl01#1
./scripts/verify.ps1
```
