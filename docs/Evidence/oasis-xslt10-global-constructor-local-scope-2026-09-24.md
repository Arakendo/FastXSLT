# OASIS XSLT 1.0 Global-Constructor Local Scope

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged `Lotus/variable_variable67#1` constructs one global result-tree
fragment from explicit text and repeated references to an earlier global
variable. Two literal-valued local variables are declared between those
references. The local declarations must neither replace nor disturb the
earlier global binding.

FastXSLT already represented this text-only XSLT 1.0 constructor as an
immutable temporary document, but its bounded text-parts compiler rejected
local declarations before it could retain the surrounding global references.

## Implemented slice

The existing XSLT 1.0 temporary-text-parts compiler now admits interleaved
static local variable declarations. Literal-valued locals are tracked in
lexical order and folded into later `xsl:value-of` text parts. A variable
reference not shadowed by a preceding local remains a global dependency and is
resolved through the existing invocation-owned global frame.

The slice preserves the existing duplicate-local diagnostic, requires each
admitted local to have a static string-literal `select`, and does not add a
general sequence-constructor evaluator. A focused runtime test proves that
global references before, between, and after two local declarations retain the
same value and that local declarations emit no result content.

## Corpus result

`Lotus/variable_variable67#1` now initializes, executes, and compares exactly.
The complete sweep moves from:

- 2,292 to 2,293 initialized cases;
- 2,240 to 2,241 successfully executed cases; and
- 2,081 to 2,082 exact expected-result matches.

Initialization failures fall from 843 to 842. Execution failures and XML
mismatches remain 52 and 76. The exact compatibility lower bound is therefore
**2,082 / 3,173 (65.62%)**.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_text_tree_preserves_global_scope_across_static_local_bindings
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/variable_variable67#1'
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
