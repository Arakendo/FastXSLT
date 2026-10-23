# OASIS XSLT 1.0 Apply-Imports Import-Subtree Range

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can `xsl:apply-imports` select only templates in the import subtree of the
current matched template while preserving imports introduced through
`xsl:include` expansion?

## Method

- Retain ordinary integer import precedence as the primary template-selection
  rank.
- Add a private minimum eligible import precedence to each compiled matched
  template.
- Rebase that floor with its owning imported program.
- Give a direct imported branch only its own nested-import range; numerically
  lower sibling branches remain ineligible.
- When an included module contributes imports, share the combined import floor
  among same-precedence declarations because inclusion composes one stylesheet
  module.
- Keep ordinary template selection and built-in rule fallback unchanged after
  the bounded imported search finds no eligible explicit rule.
- Add focused sealed-resource regressions for both sibling-branch isolation and
  an including declaration seeing an import introduced by the included module.
- Compare the current tree against commit `5252847` and rerun the unchanged
  3,173-case catalog.

## Result

The unchanged `Lotus/impincl_impincl20#1` case no longer lets the later sibling
import's `xsl:apply-imports` enter an earlier sibling branch. It falls back to
the built-in element rule, whose child dispatch correctly reaches the
principal `bar` template. The unchanged `Lotus/impincl_impincl12#1` case still
allows a principal declaration to reach the lower-precedence import introduced
through an included module.

The complete sweep initializes 2,195 cases and executes 2,138 successfully.
Exact XML-semantic matches rise from 1,988 to 1,989 and mismatches fall from 66
to 65. Initialization failures remain 940, execution failures remain 57, and
comparator-unsupported results remain 75. The strict compatibility lower bound
is `1,989 / 3,173 = 62.68%`.

## Boundaries

- The range is immutable compiled template metadata, not invocation state or a
  public stylesheet representation.
- The change narrows only `xsl:apply-imports`; ordinary template selection,
  mode matching, priority, declaration-order resolution, diagnostics, and work
  charging retain their existing paths.
- Inclusion does not create a new precedence level. Same-precedence
  declarations receive the composed module's import floor.
- An empty imported branch creates no selectable template and does not panic or
  manufacture a candidate.
- Resource acquisition remains sealed and memory-resident. No filesystem or
  network authority is added, and the local OASIS archive is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features apply_imports --no-fail-fast
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'impincl_'
./scripts/verify.ps1
```
