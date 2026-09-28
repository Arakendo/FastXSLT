# OASIS XSLT 1.0 Qualified Predicated Child Path

- Date: 2026-09-26
- Status: Verified frontier evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Implemented slice

The shared qualified-child path parser now composes statically resolved
expanded-name tests with the ordinary typed predicate representation. A path
such as `p:entry[2]` therefore retains both its compile-time namespace binding
and its positional predicate while continuing to use the established location-
path evaluator.

No runtime prefix lookup, namespace-axis representation, new evaluator, or
unbounded expression grammar was added.

## Corpus result

Unchanged Microsoft `Namespace_XPath_ScopingRules` advances past
`p1:AAA[2]` compilation. Its principal source then fails XML namespace
validation because it binds the non-`xml` prefix `ws` to the reserved XML
namespace URI. FastXSLT retains that parser rejection rather than weakening
Namespaces in XML conformance for a historical normal-result fixture.

The case therefore moves from `XPST0003` to visible `FXXM0002` and receives no
exact-result credit. Overall conserved totals remain **2,153 / 3,173 exact
matches (67.85%)**, 2,361 initialized, and 2,310 successfully executed.

## Verification

```powershell
cargo test -p fastxslt --all-features qualified_child_path_retains_ordinary_position_predicates
./scripts/measure-oasis-xslt10.ps1 -TraceCase Namespace_XPath_ScopingRules
./scripts/verify.ps1
```
