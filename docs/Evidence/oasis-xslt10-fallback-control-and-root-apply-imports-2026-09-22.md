# OASIS XSLT 1.0 Fallback Control and Root Apply-Imports

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Can FastXSLT preserve XSLT 1.0 namespace-control scope on unavailable
extension fallback content and retain the matched-rule context required when a
principal `match="/"` template executes `xsl:apply-imports`?

## Method

- Admit `xsl:exclude-result-prefixes` on an XSLT 1.0 `xsl:fallback` only as an
  ignored control attribute.
- Apply namespace exclusion declarations only from stylesheet roots and
  literal result elements. Do not let declarations on XSLT instructions or
  extension elements alter descendant literal-result namespaces.
- Preserve the compact direct root-template representation normally used by
  the private engine.
- When a principal root template contains `xsl:apply-imports`, materialize that
  root as an ordinary matched rule at principal import precedence so runtime
  selection supplies the rule identity needed for lower-precedence dispatch.
- Exercise both conventional and simplified imported root stylesheets with
  focused runtime tests, then rerun the unchanged 3,173-case OASIS catalog.

## Result

The Microsoft `BVTs_bvt020` case now initializes and executes. Its extension
element and `xsl:fallback` namespace exclusions are correctly ignored, and the
fallback literal result retains the in-scope namespace. The case reaches a
later indentation mismatch rather than being credited as exact.

Five previously unavailable executions now complete after the principal root
retains matched-rule context. Four compare exactly and the fifth is
`BVTs_bvt020`. Conserved totals are 2,150 initialized cases, 985 initialization
failures, 2,094 successful executions, 56 execution failures, 1,942 exact
XML-semantic matches, and 62 mismatches. The strict compatibility lower bound
is `1,942 / 3,173 = 61.20%`.

## Boundaries

- Namespace-control attributes remain scoped by their standard element kinds;
  this does not make arbitrary instruction or extension attributes effective.
- FastXSLT still executes no extension implementation and grants no extension
  callback or resource authority.
- Root templates without `xsl:apply-imports` retain the existing compact direct
  representation.
- The representation change is private and creates no public plan or template
  identity contract.
- The remaining `BVTs_bvt020` indentation mismatch is not counted as a pass.

## Reproduction

```powershell
cargo test -p fastxslt --all-features declared_extension_elements_compile_only_standard_fallback_content
cargo test -p fastxslt --all-features principal_root_template_preserves_matched_rule_context_for_apply_imports
cargo test -p fastxslt --all-features apply_imports_executes_a_simplified_imported_root_template
./scripts/measure-oasis-xslt10.ps1 -TraceCase BVTs_bvt020
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
