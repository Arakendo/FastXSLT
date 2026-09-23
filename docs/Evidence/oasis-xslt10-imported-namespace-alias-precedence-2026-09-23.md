# OASIS XSLT 1.0 Imported Namespace-Alias Precedence

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

When a principal stylesheet and its single imported module declare competing
`xsl:namespace-alias` mappings for the same stylesheet namespace, can the
higher-precedence principal mapping govern literal result constructors from
both modules without trying to reverse an already-applied lower mapping?

## Method

- Compile and validate the namespace-alias declarations from both modules
  before compiling the imported program's constructors.
- When the principal has aliases and the bounded graph has one direct import,
  exclude the imported alias declarations from that module's ordinary local
  rewrite pass.
- Build one effective alias set by retaining imported mappings not shadowed by
  the principal and overlaying every higher-precedence principal mapping.
- Apply that effective set exactly once to the imported program.
- Keep the existing single-document path and module resource authority
  unchanged.
- Add a focused sealed-resource regression in which the principal maps a
  literal namespace to the XSLT namespace while the imported module maps the
  same namespace elsewhere; both literal results must use the principal map.

## Result

The unchanged `Microsoft/Namespace-alias_NSAlias_In_Import#1` case now emits
both literal result elements in the principal alias's XSLT namespace and
compares exactly.

The complete 3,173-case sweep still initializes 2,195 cases and executes 2,138
successfully. Exact XML-semantic matches rise from 1,987 to 1,988; mismatches
fall from 67 to 66. Initialization failures remain 940, execution failures
remain 57, and comparator-unsupported results remain 75. The strict
compatibility lower bound is `1,988 / 3,173 = 62.65%`.

## Boundaries

- The composition is admitted for one direct imported module when the
  principal supplies namespace aliases. Multiple sibling imports and nested
  alias-precedence graphs are not inferred from this result.
- Every excluded imported alias declaration is still parsed and statically
  validated before composition.
- The implementation never attempts to reverse a previously rewritten name;
  doing so could corrupt a literal name that legitimately used the lower
  alias's result namespace.
- Namespace-alias declarations and the effective mapping remain private
  compiler state.
- No filesystem or network access is added, and the archive remains locally
  acquired rather than redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features principal_namespace_alias_outranks_imported_alias_for_all_literal_results
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/Namespace-alias_NSAlias_In_Import#1
./scripts/verify.ps1
```
