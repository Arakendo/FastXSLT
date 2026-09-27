# OASIS XSLT 1.0 Temporary-Tree `substring()`

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

FastXSLT already owned XSLT 1.0 string conversion for invocation-local
temporary trees and the shared XPath `substring()` implementation. The value
compiler nevertheless admitted only literal and location-path substring
operands. Unchanged Microsoft case
`RTF_RTF_to_string_substring_function#1` therefore stopped at generic
`FXXP1001` before either existing operation could run.

## Implemented slice

The XSLT 1.0 value compiler now retains a typed direct-variable substring plan
with one statically parsed finite start and optional length. Runtime resolves
the variable through the existing compatibility string conversion, including
temporary-document descendant-text traversal, charges the function operation,
and invokes the shared codepoint-aware substring evaluator.

The plan is compatibility-only and deliberately narrow. It does not add a
general function-call grammar, dynamic position operands, modern implicit
conversion, cross-invocation tree retention, or another XPath evaluator.

A focused runtime test covers both two- and three-argument calls over a global
temporary tree. Compiled retention accounting includes the owned variable
name; the temporary value remains invocation-owned.

## Corpus result

The unchanged Microsoft case now serializes `<out>Some</out>` and matches its
reference result. The complete sweep advances from **2,329 to 2,330
initialized**, from **2,282 to 2,283 successfully executed**, and from **2,117
to 2,118 / 3,173 exact matches (66.75%)**. Initialization failures fall from
841 to 840; execution failures, mismatches, exclusions, and comparator gaps are
unchanged.

The complete three-case `XPath-Expression` category is now successfully
executed and exactly matched. This category closure is a local corpus fact, not
a general XPath 1.0 conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_substring_converts_a_global_temporary_tree_to_string
./scripts/measure-oasis-xslt10.ps1 -TraceCase RTF_to_string_substring_function
./scripts/verify.ps1
```
