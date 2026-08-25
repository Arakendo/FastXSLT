# W3C Suite Catalog Inventory

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| FastXSLT baseline | `f5e6064` |
| Method | `scripts/inventory-conformance-sources.ps1` |
| Decision pressure | AR-0001 initial standards profile and harness scope |

## Inputs

| Suite | Revision | Root catalog |
| --- | --- | --- |
| QT3 | `83993587711dbd5c18ed846385ec37d079d6e492` | `vendor/qt3tests/catalog.xml` |
| XSLT 3.0 | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` | `vendor/xslt30-test/catalog.xml` |

The submodule integrity check passed before catalog traversal. Both worktrees
were clean and at the revisions above.

## Method

The PowerShell inventory uses `System.Xml.XmlReader` in forward-only mode. DTD
processing is prohibited, the XML resolver is absent, and comments, processing
instructions, and insignificant whitespace are ignored.

For each suite it:

1. reads direct `test-set` references from the root catalog;
2. normalizes each referenced path without resolving a URI or external entity;
3. rejects missing files and duplicate root references;
4. counts `test-case` elements in every distinct referenced test-set document;
   and
5. emits deterministic JSON with the pinned revision and structural totals.

Command:

```text
./scripts/inventory-conformance-sources.ps1
```

## Results

| Suite | Root references | Distinct test sets | Test cases | Missing sets | Duplicate references |
| --- | ---: | ---: | ---: | ---: | ---: |
| QT3 | 428 | 428 | 31,821 | 0 | 0 |
| XSLT 3.0 | 234 | 234 | 14,600 | 0 | 0 |

These totals reproduce the catalog counts reported by the TS XSLT peer at the
same revisions.

## Limitations

- The inventory does not execute a test or establish FastXSLT support.
- It does not yet classify language editions, dependencies, optional features,
  environments, assertions, serialization requirements, or expected errors.
- It does not validate the catalogs against their schemas.
- It does not establish a denominator for a conformance percentage.
- It does not inventory a separate XSLT 1.0 suite candidate, so AR-0001's
  smaller-profile alternative still lacks equivalent suite evidence.

## Result

The two modern W3C suites are present, structurally complete relative to their
root catalogs, and reproducibly countable. AR-0001 may use them as known
candidate evidence, but must still select the standards profile and define
dependency-aware selection before a harness reports supported, unsupported,
failed, or harness-error cases.
