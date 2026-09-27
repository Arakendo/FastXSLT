# OASIS XSLT 1.0 Namespace-Wildcard Whitespace Policy

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related reviews: AR-0016, AR-0019
- Related decision: ADR-0012

## Pressure

After same-module exact and wildcard composition was admitted, unchanged Lotus
case `whitespace07` was the sole remaining `FXST1043` case. It requires the
XSLT 1.0 `prefix:*` whitespace NameTest: whitespace-only text must be stripped
from elements in the prefix's statically resolved namespace while elements in
other namespaces remain unchanged.

## Implemented slice

The private compiled whitespace policy now retains disjoint strip and preserve
namespace-URI sets alongside exact expanded-name overrides and the wildcard
default. Matching applies the XSLT NameTest priority order:

1. exact expanded name;
2. namespace wildcard; and
3. unqualified `*` wildcard.

Later declarations replace earlier actions at the same priority. Namespace
prefixes are resolved during stylesheet compilation, so the invocation view
compares namespace identity rather than source or stylesheet prefix spelling.
Both the complete safe reference and the visibility view use the same private
predicate, preserve visible node identity and provenance, retain inherited
`xml:space` behavior, and charge the existing XDM construction work.

This does not admit general mixed whitespace-policy composition across
include/import precedence, schema-aware whitespace semantics, or a public
source-view abstraction.

## Verification

A focused compiler test proves `n:*` is retained as the namespace URI `urn:n`.
A focused differential runtime test proves an exact preserve rule overrides a
namespace strip rule, a sibling in the selected namespace is stripped, and a
sibling in another namespace remains preserved. The complete reference and
visibility view agree while the prepared source remains unchanged.

The unchanged `Lotus/whitespace_whitespace07#1` case leaves `FXST1043`,
executes, and compares exactly. The complete sweep therefore moves from 2,301
to 2,302 initialized cases and from 2,249 to 2,250 successful executions.
Exact expected-result matches rise from 2,088 to **2,089 / 3,173 (65.84%)**.
The `FXST1043` frontier is eliminated.

## Reproduction

```powershell
cargo test -p fastxslt --all-features compiles_strip_all_and_exact_expanded_name_whitespace_policies
cargo test -p fastxslt --all-features xslt10_namespace_wildcard_whitespace_rules_preserve_exact_name_priority
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
