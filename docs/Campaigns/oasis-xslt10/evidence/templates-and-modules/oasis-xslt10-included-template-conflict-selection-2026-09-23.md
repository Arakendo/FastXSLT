# OASIS XSLT 1.0 Included Template Conflict Selection

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can template rules contributed through `xsl:include` participate in ordinary
XSLT conflict selection without inventing a module-specific selection path or
losing textual declaration order?

## Method

- Preserve an included module's compiled template rules as one ordered block at
  the textual position of its `xsl:include` declaration.
- Retain each rule's existing import precedence and explicit/default priority.
- When both modules contain a document rule, materialize their formerly
  shortcut `root_template` values as ordinary `MatchPattern::Document` rules at
  their original declaration positions.
- Use the existing runtime selector, which ranks matching rules by import
  precedence and priority and uses later declaration order for the XSLT 1.0
  recovery choice.
- Keep the existing multiple-match error policy available for modes/profiles
  that require an error rather than recovery.

## Result

The `FXST1020` duplicate-root and `FXST1021` duplicate-included-match
frontiers disappear. Ten additional unchanged OASIS cases initialize, execute,
and compare exactly, including the five include/output cases exposed by the
preceding included-output tranche.

The complete sweep now initializes 2,181 cases, reports 954 initialization
failures, executes 2,125 successfully, and reports 56 execution failures.
Exact XML-semantic matches rise from 1,969 to 1,979, with 65 mismatches. The
strict compatibility lower bound is `1,979 / 3,173 = 62.37%`.

## Boundaries

- This relies only on already-retained import precedence, template priority,
  source location, and ordered module placement; it adds no second selector.
- The result does not solve `xsl:apply-imports` import-subtree ancestry. A flat
  precedence value is still insufficient for that separate semantic rule.
- The result does not admit textual expansion of multiple interleaved includes;
  the three documented `FXST1029` cases remain outside the compiled-program
  composition seam.
- No compiled representation or selection mechanism becomes public.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features included_root_template_competes_in_textual_declaration_order
cargo test -p fastxslt --all-features principal_template_after_two_includes_wins_same_precedence_conflict
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
