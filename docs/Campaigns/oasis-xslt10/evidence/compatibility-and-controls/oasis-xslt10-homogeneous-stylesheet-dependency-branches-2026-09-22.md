# OASIS XSLT 1.0 Homogeneous Stylesheet Dependency Branches

Date: 2026-09-22  
Status: Local implementation and compatibility evidence

## Question

Can FastXSLT recursively compose a bounded nested module that has two sibling
includes or two sibling imports by reusing its established two-program
composition paths?

## Method

- Retain the existing loader limits of depth two, five modules, and 1 MiB.
- Recursively compile leaf modules and single dependency edges.
- At a two-child branch, continue only when both edges have the same kind and
  compose the two compiled child programs through the existing include/include
  or import/import helper.
- Leave mixed include/import sibling branches explicit because flattening an
  included module's imports requires a separate precedence decision.
- Add a focused sealed-resource regression in which an included module includes
  two leaf modules providing distinct named templates.
- Rerun the unchanged 3,173-case OASIS catalog and trace the `FXST1027` frontier.

## Result

The focused regression produces `LEFTRIGHT` through a principal include, a
nested two-include branch, and two leaf programs. Six of the eight remaining
`FXST1027` cases now initialize and execute: five compare exactly, while
`Lotus/impincl_impincl20#1` reaches a visible mismatch.

Two cases remain at the explicit boundary. `Lotus/impincl_impincl23#1` requires
the import precedence of a root import to compose with an included module's own
import. `Microsoft/Import_Import_IncludeBeforeImportInStylesheet#1` deliberately
places an include before an import and expects a static error. Those are not
treated as the same problem merely because both have mixed dependency kinds.

The complete sweep initializes 2,170 cases, reports 965 initialization failures,
executes 2,114 successfully, and reports 56 execution failures. Exact
XML-semantic matches reach 1,967, with 65 mismatches. The strict compatibility
lower bound is `1,967 / 3,173 = 61.99%`.

## Boundaries

- Nested branches remain bounded by the existing loader policy.
- This slice admits homogeneous sibling edges only; mixed include/import
  flattening and precedence are not inferred.
- The existing program-composition implementation remains the semantic owner;
  the resource compiler only supplies recursively compiled child programs.
- Resource authority, sealed-snapshot execution, diagnostics, and the absence
  of engine-owned I/O are unchanged.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features nested_include_branch_reuses_two_program_composition
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier unsupported/FXST1027/FXST1027
./scripts/verify.ps1
```
