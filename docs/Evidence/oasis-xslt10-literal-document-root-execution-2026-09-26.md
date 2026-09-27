# OASIS XSLT 1.0 Literal `document()` Root Execution

- Date: 2026-09-26
- Status: Verified corpus and focused-regression evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related decisions and reviews: ADR-0002, AR-0014, and AR-0019

## Pressure

Four unchanged OASIS cases select a literal, non-empty `document()` call
directly from `xsl:apply-templates` or `xsl:for-each`. The supplemental bytes
were already admitted into each case's sealed snapshot, but the compiler could
not represent the selected external document root. An initial apply-template
slice also exposed that the shared node-selection helper must never receive a
multi-document selection it cannot represent: the three `for-each` cases were
caught as execution panics by the conserved measurement harness.

## Implemented slice

The compiler now retains a direct literal `document('reference')` or
`document("reference")` selection as a typed external-document-root selection.
At execution:

- resolution uses only the invocation's immutable sealed snapshot and applies
  host denial before membership disclosure;
- the supplemental document is parsed and constructed under existing work
  budgets, then shared only within the invocation through an `Arc<Document>`;
- `xsl:apply-templates` performs ordinary mode-aware template selection with
  the external document root as focus;
- `xsl:for-each` executes its body once with that same external root, including
  the existing charged sort-control path; and
- each external document receives its own invocation-local document-rooted
  match cache.

The slice deliberately rejects non-empty source-node variables or parameters,
source-node globals, and stylesheet whitespace filtering because raw `NodeId`
values and source visibility are not yet qualified by document identity in
those paths. The variable guard is intentionally conservative even when a
particular body does not consume the in-scope binding. Dynamic references,
empty `document('')`, second base arguments, path tails, and arbitrary
mixed-document node sequences remain unsupported. No filesystem or network
authority is inferred from the URI spelling.

A focused regression proves mode-aware template dispatch reads the external
source rather than the principal input. The three former panic cases now all
return structured unsupported `FXRT1017` outcomes instead of internal failure.

## Measurement result

Unchanged case `Microsoft/Template_DocumentFNTakesStringParam#1` now compares
exactly. The complete conserved sweep moves from 2,335 to **2,339 initialized**,
from 2,287 to **2,288 successfully executed**, and from 2,122 to **2,123 /
3,173 exact matches (66.91%)**. Initialization failures fall from 835 to 831;
execution failures rise from 48 to 51 because the former panic cases now have
an explicit disposition. Visible mismatches remain 69.

`Lotus/idkey_idkey18#1`, `Microsoft/Keys__91834#1`, and
`Microsoft/Keys__91835#1` now stop at `FXRT1017`. They are useful pressure for
document-qualified local source-node values and per-document key indexes; this
tranche does not infer those capabilities merely because parsing the external
document root is possible.

This is compatibility evidence from a locally acquired archival corpus, not an
XSLT 1.0 conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_apply_templates_to_literal_document_root_uses_external_source_context
cargo test -p fastxslt --all-features xslt10_literal_document_root_rejects_unqualified_source_node_variables
./scripts/measure-oasis-xslt10.ps1
```
