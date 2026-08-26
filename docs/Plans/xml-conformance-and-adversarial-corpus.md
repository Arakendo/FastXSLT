# XML Conformance and Adversarial Corpus

| Field | Value |
| --- | --- |
| Status | In Progress |
| Opened | 2026-08-25 |
| Owner | FastXSLT maintainers |
| Target | AR-0008 standards-derived XML evidence and separate hostile-input evidence |
| Related reviews | AR-0001, AR-0008, AR-0010, AR-0011 |
| Depends on | XML/Namespaces edition selection and external-suite rights review |

## Purpose

Use accumulated XML standards cases to test parser-boundary correctness while
building a separate first-party adversarial corpus for bounded termination.
Neither activity creates an XSLT conformance claim or a performance benchmark.
AR-0011 owns the shared inventory/disposition/reporting principles; this plan
sequences XML-specific acquisition and execution evidence.

## Corpus responsibilities

| Family | Question | Authority | Result categories |
| --- | --- | --- | --- |
| XML conformance | Does the parser boundary implement a named XML/Namespaces profile? | Unmodified upstream cases plus FastXSLT selection overlay | selected, excluded profile, unsupported, harness unsupported, pass, fail |
| Adversarial | Does hostile-but-admitted work stop through explicit structural/work limits? | First-party generated/minimized fixtures with provenance | bounded success, named limit, cancellation, unexpected failure |
| Performance | What does correct work cost for a named workload and host? | Benchmark manifests and correctness-gated fixtures | latency, throughput, allocation/retention, peak memory |

## Phase 0: Candidate inspection

**Status:** Complete.

- [x] Reconcile Monday's recommendations with existing QT3/XSLT30 pins.
- [x] Download the official W3C XML 20130923 archive to ignored storage.
- [x] Record its SHA-256, file inventory, catalog categories, and harness risks.
- [x] Identify older mixed contributor notices and avoid repository admission.

## Phase 1: Acquisition and classifier

**Status:** Pending.

- [ ] Decide whether the suite remains optional local input or can be
  redistributed after a focused rights review.
- [ ] Provide a bounded acquisition command that verifies the exact archive
  digest before extraction and never treats a URI/path as engine authority.
- [ ] Inventory the 21 declared local catalog fragments without enabling ambient
  DTD or entity resolution.
- [ ] Retain case ID, URI, type, recommendation, version, edition, namespace
  mode, entity mode, output assertion, and collection identity.
- [ ] Reject duplicate identities, missing fixture paths, root escapes, unknown
  metadata, and catalog/fixture mutations as harness failures.

## Phase 2: AR-0008 subset

**Status:** Pending AR-0001 XML edition selection.

- [ ] Select well-formed/not-well-formed and Namespaces cases applicable to the
  chosen editions and nonvalidating authority policy.
- [ ] Exclude or separately classify validation, external-entity, unsupported
  encoding, XML 1.1, and canonical-information assertions as appropriate.
- [ ] Compare parser-adapter accept/reject behavior while preserving upstream
  identity and structured FastXSLT diagnostics.
- [ ] Record selected, excluded, unsupported, passed, failed, and harness-error
  counts without an unqualified percentage.
- [ ] Feed accepted applicable documents into XDM construction where the case
  asserts information FastXSLT actually owns.

## Phase 3: First-party adversarial XML/XDM set

**Status:** Pending representative failures and limits.

- [ ] Generate or minimize cases for admitted bytes, deep nesting, attribute and
  namespace growth, sibling/node counts, giant text, and late malformed input.
- [ ] Record generator/seed, exact bytes, expected structural or work domain,
  and whether partial semantic/result state is observable.
- [ ] Exercise cancellation at named phase checks and retain request/domain
  identity.
- [ ] Keep stress expectations independent from standards conformance and from
  production default budgets.

## Exit criteria

- The XML suite is either reproducibly available under an explicit local-only
  boundary or deliberately admitted with reviewed rights and provenance.
- One edition-aware subset runs without ambient external-resource access.
- Results distinguish selection, unsupported behavior, harness failure, and
  engine failure.
- First-party hostile-input cases prove named bounds without being reported as
  standards cases or benchmark samples.

## Next slice

Resolve the intended XML/Namespaces editions through AR-0001 and review whether
local-only hash-verified acquisition is sufficient for contributors and CI.
