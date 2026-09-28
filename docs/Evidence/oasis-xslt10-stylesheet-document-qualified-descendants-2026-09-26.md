# OASIS XSLT 1.0 Stylesheet-Document Qualified Paths

- Date: 2026-09-26
- Status: Verified semantic and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Implemented slice

Literal-document `xsl:for-each`, `xsl:apply-templates`, and `xsl:copy-of`
selection now admits:

```xpath
document('')//prefix:local
document('')/prefix:local/prefix:local[@attribute='literal']
```

The empty reference resolves against the static stylesheet identity through the
sealed snapshot, so it selects the stylesheet document without ambient file or
network access. The descendant name test is expanded at compile time from the
stylesheet namespace context and retained as an `ExpandedName`. Execution uses
the existing invocation-owned dynamic-document cache, whitespace visibility,
charged iterative traversal, and external focus machinery.

For `xsl:copy-of`, a literal document reference may now carry the existing
typed `LocationPath` rather than being reduced to a bespoke descendant string.
Qualified child name tests are expanded at compile time; predicates, document
order, work charging, whitespace visibility, and subtree copying continue
through the shared path evaluator and result constructor. An empty reference
retains the instruction's declaring module as its base, including when the
instruction came from an included or imported module.

The slice does not add dynamic document arguments, two-argument `document()`,
namespace-axis nodes, wildcard namespace tests, cross-document globals, or a
public node-provider boundary.

## Evidence

The focused regression proves that `document('')//ped:test` selects a top-level
stylesheet data element by expanded name and excludes unrelated names.
The private `DocumentRootReference` now retains the optional descendant as an
expanded name rather than a local-name string, so copying and identity checks
cannot silently discard the statically resolved namespace.

Unchanged Lotus `namespace_namespace20` now executes two qualified stylesheet-
document descendant selections, copies the selected elements with their
namespace scope, and compares exactly. Unchanged Lotus `idkey_idkey50` advances
past the expression grammar into the existing `FXRT1017` guard because it also
requires node-valued cross-document global state and key evaluation; it is not
credited as a pass.

Unchanged Microsoft `XSLTFunctions_DocumentFuncWithEmptyArg` copies the
qualified top-level stylesheet data element with its namespace scope and
matches the archival reference exactly.

Unchanged Lotus `mdocs09`, `mdocs12`, and `mdocs13` select a qualified
stylesheet data element through a literal attribute predicate and compare
exactly. The latter two prove module-local empty-reference resolution across
include and import. Unchanged Lotus `copy27` also compares exactly after its
wildcard, qualified-template, attribute-predicate, and final `node()` path is
evaluated by the same typed location-path backend.

## Conserved result

Across the qualified stylesheet-document slices, the sweep moves from 2,350
to **2,357 initialized** cases, from 2,300 to **2,306 successful executions**,
and from 2,143 to **2,149 exact matches (67.73%)**. Initialization failures fall
from 820 to 813; execution failures rise from 50 to 51 because `idkey50`
reaches its later explicit ownership boundary. Expected-error credit remains
423 / 431, comparator gaps remain 67, and visible XML mismatches remain zero.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_for_each_selects_qualified_descendants_from_the_stylesheet_document
cargo test -p fastxslt --all-features xslt10_copy_of_qualified_stylesheet_document_descendants_uses_expanded_names
cargo test -p fastxslt --all-features xslt10_copy_of_qualified_stylesheet_document_path_applies_predicate
./scripts/measure-oasis-xslt10.ps1 -TraceCase namespace_namespace20
./scripts/measure-oasis-xslt10.ps1 -TraceCase idkey_idkey50
./scripts/measure-oasis-xslt10.ps1 -TraceCase XSLTFunctions_DocumentFuncWithEmptyArg
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs_mdocs09
./scripts/verify.ps1
```
