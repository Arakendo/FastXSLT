# OASIS XSLT 1.0 Post-74% Frontier and DTD Inventory

## Purpose

Record the first work after the 74.00% checkpoint and identify why the next
percentage point cannot responsibly be obtained by continuing to admit isolated
syntax shapes without reviewing XML DTD/entity semantics.

## Conserved baseline

The complete hash-verified OASIS XSLT/XPath 1.0 Committee Draft 04 denominator
remains 3,173 catalog cases. Commit `d9e12c9f` establishes 2,348 exact expected
result comparisons (74.00%). Reaching 75.00% requires 2,380 exact comparisons,
or 32 additional cases.

## Post-checkpoint experiments

Two safe private capabilities were implemented and focused-tested:

- declared ISO-2022-JP XML input is incrementally transcoded to UTF-8 while
  parser offsets remain mapped to the immutable admitted byte stream; malformed
  input fails closed and no declaration means no encoding guess; and
- `xsl:processing-instruction/@name` can use the existing bounded
  attribute-value-template representation, with execution-time NCName/`xml`
  validation and no extra result-node budget charge.

The complete sweep remains at 2,348 exact comparisons. The synthetic valid
ISO-2022-JP case proves the decoder boundary, while the sampled archival input
contains malformed sequences and remains a structured initialization failure;
FastXSLT does not replace invalid bytes merely to make the case execute. The two
catalog stylesheets containing a dynamic processing-instruction target also
contain earlier unsupported `current()` and stylesheet-document AVTs. These
capabilities are retained as truthful breadth but are not counted as progress.

## DTD frontier

The complete sweep reports these directly blocked standard-operation cases:

| First failure | Cases |
| --- | ---: |
| Principal source DTD denied | 87 |
| Principal stylesheet DTD denied | 30 |
| Total direct DTD-denial frontier | 117 |

Related cases beyond that direct frontier exercise `id()` over DTD-typed
attributes and `unparsed-entity-uri()` over entity declarations. Accepting and
discarding a `DOCTYPE` event would therefore be semantically wrong even where
it allows parsing to continue.

An initial physical inventory of all XML/XSL files in the local archive found:

| Property | Files |
| --- | ---: |
| XML/XSL files inspected | 5,796 |
| Files containing `DOCTYPE` | 110 |
| Internal subsets | 101 |
| `SYSTEM` identifiers | 36 |
| `PUBLIC` identifiers | 1 |
| Entity declarations | 47 |
| Attribute-list declarations | 68 |
| ID-typing candidates | 32 |
| Notation declarations | 26 |

These properties overlap. Only two files were simple internal-subset candidates
containing element declarations without entity, attribute-list, notation,
external-identifier, or general-reference pressure. Consequently, an
"accept inert DTD and ignore it" shortcut cannot supply the 32 cases needed for
75% and would not address the actual semantics dominating the corpus.

## Architectural consequence

AR-0008 deliberately denies DTDs and requires an authority/security review
before later support. AR-0025 now owns that investigation. It keeps denial as
the default and admits only a safe bounded internal-subset reference experiment
after case-level inventory. External identifiers remain identities, not
authority, and any later external dependency must resolve exclusively through
explicitly admitted sealed resources.

## Non-claims

- This evidence does not raise the exact-match numerator above 2,348.
- The 117 directly blocked cases are a pressure count, not an expected pass
  count.
- The physical-file inventory is not yet the required case-by-case declaration
  inventory.
- No DTD, entity, validation, typed-ID, external-resource, or XSLT 1.0
  conformance support is selected by this record.
