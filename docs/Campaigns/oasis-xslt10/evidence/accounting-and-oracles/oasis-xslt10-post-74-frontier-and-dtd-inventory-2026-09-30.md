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

The first two post-checkpoint capabilities leave the sweep at 2,348 exact
comparisons. The synthetic valid
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
| Stylesheet input DTD denied (28 principal, 2 dependency) | 30 |
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

The reproducible catalog inventory in
`scripts/inventory-oasis-xslt10-dtd.ps1` separately follows each standard test
case's selected principal inputs. Repeated references remain repeated because
they represent distinct catalog pressure:

| Property | Standard principal inputs |
| --- | ---: |
| Inputs containing `DOCTYPE` | 153 |
| Principal sources | 125 |
| Principal stylesheets | 28 |
| Internal subsets | 146 |
| External identifiers | 110 |
| Entity declarations | 19 |
| Attribute-list declarations | 55 |
| ID-typing candidates | 27 |
| Notation declarations | 2 |

This catalog inventory is intentionally broader than the 117 current direct
failures: some cases are excluded or fail earlier for another classified
reason, and multiple cases reuse the same physical input. Correlating every
direct failure with declaration-level semantic dependencies remains required
before implementation.

The measurement runner now also classifies the exact 117 current standard-case
frontiers at the point where the engine reports `dtd-forbidden`. These counts
are therefore a conserved failure denominator rather than a broader input
inventory:

| Property | Direct frontier cases |
| --- | ---: |
| Principal source | 87 |
| Stylesheet input (28 principal, 2 dependency) | 30 |
| Internal subset | 113 |
| External identifier | 94 |
| Attribute-list declaration | 41 |
| Explicit default-attribute candidate | 27 |
| Any entity declaration | 17 |
| Internal general-entity candidate | 13 |
| External general-entity candidate | 5 |
| Typed-ID candidate | 11 |
| Parameter entity / notation / unparsed entity candidate | 0 |
| XML expected-result comparator | 115 |
| Manual expected-result comparator | 2 |

Properties overlap. In particular, most internal subsets also name an external
identifier, so "internal subset" does not imply that an internal-only parser is
sufficient. After the inventory, the private parser added syntax-validating,
non-validating `ELEMENT` declarations to the bounded internal-general-entity
profile. Exactly two resources now parse: `Lotus/select_select73`, which uses
one internal character-data entity, and `Microsoft/Elements__89108`, whose
declaration is semantically inert to the selected transform. Both unchanged
cases initialize, execute, and compare exactly through the measurement-only
profile. The other 115 resources remain explicitly unsupported. The original
117-case denominator, its 87/30 role split, and every declaration-property
count remain conserved rather than disappearing when the failure diagnostic
changes.

The 21 still-blocked cases without external identifiers are not one ATTLIST
tranche. Ten source cases require DTD-typed ID metadata, two source cases use
default attribute values, and nine stylesheet cases obtain fixed/defaulted
namespace or version attributes from the DTD. Those stylesheet attributes
must participate before namespace resolution, so the current start-event
adapter cannot add them afterward without changing the meaning of the parsed
stylesheet.

## Architectural consequence

AR-0008 deliberately denies DTDs and requires an authority/security review
before later support. AR-0025 now owns that investigation. It keeps denial as
the default and admits only a safe bounded internal-subset reference experiment
after case-level inventory. External identifiers remain identities, not
authority, and any later external dependency must resolve exclusively through
explicitly admitted sealed resources.

## Non-claims

- The bounded reference experiment raises the measured lower bound to 2,350 /
  3,173 (74.06%), but production DTD denial remains unchanged.
- The 117 directly blocked cases are a pressure count, not an expected pass
  count.
- The measurement runner now conserves declaration-property counts for the
  exact direct frontier, but dependency relevance and expected serialization
  method still require case-level interpretation.
- No DTD, entity, validation, typed-ID, external-resource, or XSLT 1.0
  conformance support is selected by this record.
