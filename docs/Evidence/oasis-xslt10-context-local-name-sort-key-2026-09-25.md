# OASIS XSLT 1.0 Context `local-name()` Sort Key

- Date: 2026-09-25
- Status: Verified shared-semantic and frontier evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The ordinary value-expression compiler already supported `local-name()` and
`local-name(.)`, but the private `xsl:sort` compiler admitted only the lexical
`name()` counterpart. Unchanged Xalan case `copy51` therefore stopped at the
sort key before reaching its actual namespace-axis and namespace-node-copy
semantics.

## Implemented slice

`xsl:sort` now retains a typed context-local-name key. Evaluation charges one
XPath node visit and returns the context node's expanded-name local part, or
the empty string for a nameless node. It does not reuse the lexical QName
operation, so a prefix cannot affect local-name ordering.

A focused runtime regression sorts prefixed and unprefixed source elements
whose lexical-QName order differs from their local-name order. The ordinary
stable multi-key sort machinery, data-type conversion, ordering, budgets, and
cancellation remain unchanged.

## Corpus disposition

The unchanged complete OASIS sweep remains **2,329 initialized**, **2,282
successfully executed**, and **2,117 / 3,173 exact matches (66.72%)**. The
generic function-rooted-path frontier falls from 53 to 52. `copy51` now reaches
the separately visible `namespace::*` selection boundary; FastXSLT does not
yet represent XPath namespace nodes, so the case receives no pass credit.

The related doubtful archival case `impincl27` was also inspected. Its
scheme-bearing `file:fragments/...` import is not treated as an ordinary
case-relative reference. Creating that mapping would conflate logical identity
with ambient filesystem behavior contrary to AR-0014, so it remains a visible
missing-resource result rather than a harness alias or hidden file access.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_sort_orders_for_each_and_apply_templates_with_stable_multiple_keys
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/copy_copy51#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/impincl_impincl27#1'
```
