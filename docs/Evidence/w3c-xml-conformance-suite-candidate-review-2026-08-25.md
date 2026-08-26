# W3C XML Conformance Suite Candidate Review

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Candidate | W3C XML Conformance Test Suite 20130923 |
| Canonical release | `https://www.w3.org/XML/Test/xmlts20130923.zip` |
| Archive SHA-256 | `F9510B3532926E1B4C2E54855B021E4B8A66EC98A5337DCF4FF07E8A41968DEB` |
| Local disposition | Inspected under ignored `target/`; not admitted or redistributed |
| Informs | AR-0008 and XML corpus plan |

## Why this candidate

The official W3C XML test page identifies the 20130923 archive as its latest
dated release and describes the suite as covering XML 1.0/1.1 editions and
Namespaces in XML. This directly pressures AR-0008's unresolved name,
declaration, character, namespace, document-structure, and encoding behavior.

The archive is not a Git repository and therefore cannot use the same immutable
gitlink workflow as QT3 and XSLT30. Reproducibility would require retaining the
canonical release URL, archive digest, extraction rules, and catalog inventory.

## Local inspection

The archive was downloaded only to ignored workspace storage, hashed, expanded,
and inspected without executing a fixture or resolving its external entities.

| Observation | Value |
| --- | ---: |
| Archive bytes | 1,574,648 |
| Expanded files | 3,386 |
| Expanded file bytes | 2,722,978 |
| Catalog cases | 2,586 |
| Valid | 812 |
| Invalid (validity-constraint failures) | 242 |
| Not well formed | 1,499 |
| Optional error | 33 |
| Cases with canonical `OUTPUT` metadata | 432 |
| Cases with edition metadata | 696 |
| Cases with version metadata | 669 |

Case counts came from the 21 catalog fragments referenced by the root catalog's
declared entities. Every case has a URI. The inventory did not parse or execute
the test document at that URI and is not a conformance result.

Recommendation metadata includes XML 1.0 defaults, XML 1.0 errata/edition
groups, XML 1.1, Namespaces 1.0, and Namespaces 1.1. Entity metadata includes
`none`, `general`, `parameter`, and `both`; omitted entity metadata defaults to
`none` under the supplied catalog DTD. Fourteen cases explicitly disable
namespace processing.

## Harness implications

The root catalog itself uses a DTD and external parsed entities to compose 21
catalog fragments. A secure FastXSLT harness should not enable general ambient
DTD/entity resolution just to read it. It can instead use a first-party,
integrity-checked inventory of the named local catalog fragments or an equally
bounded catalog adapter.

The case classes do not all mean “accept” or “reject” for FastXSLT's current
nonvalidating, DTD-denying experiment:

- valid cases may still require DTD declarations, entities, encodings, or
  editions outside the selected profile;
- invalid cases concern validity constraints and are generally acceptable to a
  nonvalidating processor;
- not-well-formed expectations can depend on whether external general or
  parameter entities are read;
- optional-error cases do not demand one result; and
- canonical output compares information reported by a parser, not FastXSLT's
  XSLT serializer.

Selection therefore needs edition, recommendation, namespace mode, entity mode,
case type, and assertion capability. A raw accepted/rejected percentage would
be meaningless.

## Licensing and provenance boundary

The archive predates W3C's current dual test-suite license statement. Its root
catalog and DTD preserve Sun and OASIS copyright notices including “All Rights
Reserved,” and its collections came from several contributors. W3C's current
test-suite licensing page explicitly says the newer policy does not affect an
existing suite until that suite is modified to include the statement.

Consequently this review does not conclude that the archive can be copied into
the MIT repository or redistributed with a package. A focused rights review is
required for the exact acquisition/distribution model. Local developer download
with hash verification remains the lower-risk candidate shape meanwhile.

## Disposition

**Useful, not admitted.** Preserve the URL, digest, inventory, and selection
requirements as evidence. Do not commit the archive or extracted fixtures, call
it MIT licensed, enable ambient entity resolution, or run an unclassified
denominator.

Next evidence should define a secure local acquisition/inventory mechanism and
an AR-0001-compatible subset. If redistribution rights remain unclear, keep the
suite an optional local input and place all FastXSLT selection/classification in
first-party overlays.
