# OASIS XSLT 1.0 Mixed-Module Import Precedence

Date: 2026-09-22  
Status: Local implementation and compatibility evidence

## Question

Can FastXSLT distinguish a valid import-then-include graph whose included module
contains a later import from an invalid include-before-import declaration order?

## Method

- Validate `xsl:import` ordering only for XSLT 1.0 standard stylesheet modules
  that actually declare dependencies, leaving simplified stylesheets and the
  existing XSLT 3.0 import-order behavior unchanged.
- Report `XTSE0200` when an import follows another meaningful top-level
  declaration.
- Add a program-composition path for exactly one principal import followed by
  one include when either dependency has already been compiled recursively.
- Place the earlier principal import below the minimum precedence retained by
  the later included program, while preserving all relative precedence inside
  both child programs.
- Keep other mixed sibling shapes explicit.
- Add focused sealed-resource regressions for invalid include-before-import
  ordering and for a later included import overriding an earlier principal
  import.

## Result

The invalid-order regression reports `XTSE0200` at the late `xsl:import`.
`Microsoft/Import_Import_IncludeBeforeImportInStylesheet#1` now reaches that
specific static diagnostic instead of the generic nested-module boundary.

The precedence regression selects the template from the included module's
import rather than the template from the earlier principal import. The unchanged
`Lotus/impincl_impincl23#1` likewise produces the expected `good-match` result
and becomes exact.

The complete sweep initializes 2,171 cases, reports 964 initialization failures,
executes 2,115 successfully, and reports 56 execution failures. Exact
XML-semantic matches reach 1,968, with 65 mismatches. The strict compatibility
lower bound is `1,968 / 3,173 = 62.02%`.

## Boundaries

- The executable mixed shape is exactly import-then-include with recursively
  compiled child programs; this does not admit arbitrary mixed sibling graphs.
- The remaining three `FXST1029` cases interleave principal declarations with
  two includes. They require textual include expansion to preserve declaration
  order; merging two already-completed included programs cannot represent that
  ordering and remains deliberately unsupported.
- Include-before-import remains invalid rather than being reordered or recovered.
- Existing depth, module-count, byte, cycle, authority, and snapshot limits are
  unchanged.
- The engine performs no filesystem or network I/O during compilation or
  execution.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features include_before_import_is_rejected_before_module_composition
cargo test -p fastxslt --all-features included_import_outranks_an_earlier_principal_import
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/Import_Import_IncludeBeforeImportInStylesheet#1
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/impincl_impincl23#1
./scripts/verify.ps1
```
