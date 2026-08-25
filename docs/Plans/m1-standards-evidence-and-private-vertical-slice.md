# M1 Standards Evidence and Private Vertical Slice

| Field | Value |
| --- | --- |
| Status | In Progress |
| Opened | 2026-08-25 |
| Last updated | 2026-08-25 |
| Owner | FastXSLT maintainers |
| Target | M1 standards decision and first private transform |
| Related ADRs | ADR-0001, ADR-0002 |
| Related reviews | AR-0001, AR-0003, AR-0004, AR-0007, AR-0008 |
| Related change requests | None |
| Depends on | Pinned W3C suites and representative consumer evidence |

## Purpose

Produce the evidence needed to decide FastXSLT's initial standards profile and
then implement the smallest private end-to-end transform without stabilizing a
public API, parser representation, diagnostic catalog, or streaming claim.

## Trigger and evidence

The workspace and W3C suite submodules are ready, but the engine performs no
transforms. AR-0001 blocks version and conformance claims until candidate
profiles, suites, and consumer needs are explicit. The seed `hello` golden case
provides a narrow syntax intersection suitable for architecture work after its
semantics are assigned to an accepted profile.

## Goals

- Make candidate-suite size, revision, licensing, and harness pressure
  reproducible.
- Close AR-0001 through a deliberate standards-profile ADR.
- Execute the seed transform through private XML, XDM, XPath, XSLT,
  compilation, runtime, resource, result, and diagnostic ownership.

## Non-goals

- A stable public transform API or ABI.
- An unqualified XSLT/XPath conformance percentage.
- Broad QT3 or XSLT30 execution before dependency-aware selection.
- Streaming implementation, persistent compiled artifacts, or ASP.NET binding.

## Ownership and dependency boundary

Suite inventory and selection are test-harness concerns. They may describe
standards metadata but do not own engine semantics. AR-0001 and its resulting
ADR own the admitted profile. Engine XML mechanics remain replaceable; XDM,
XPath, and XSLT meaning remain FastXSLT-owned.

## Slice 0: Reproducible suite baseline

**Status:** Complete.

- [x] Pin the same QT3 and XSLT30 revisions used by the TS XSLT peer.
- [x] Preserve licensing and acquisition provenance.
- [x] Verify initialized, clean submodule worktrees and exact revisions.
- [x] Walk both catalogs without DTD or external-resource resolution.
- [x] Retain structural counts and limitations as evidence.

Exit state: 31,821 QT3 and 14,600 XSLT30 cases are reproducibly discoverable;
none are represented as supported or executable.

## Slice 1: Standards profile disposition

**Status:** In Progress.

- [ ] Obtain representative transform families and compatibility needs from the
  first consumer.
- [x] Complete suite/harness evidence for the XSLT 1.0 alternative without
  admitting redistribution-constrained legacy material to the repository.
- [ ] Compare staged-modern and version-specific alternatives against time to
  first useful release, data-model growth, diagnostics, and migration risk.
- [ ] Define dependency-aware selection and reporting categories.
- [ ] Close AR-0001 through an accepted ADR.

Exit state: source code, documentation, and test selection can name one initial
profile without ambiguity.

## Slice 2: Private golden vertical behavior

**Status:** In Progress as a private, version-neutral experiment; public
semantics remain pending AR-0001 disposition.

- [x] Select a leading XML parser for private evaluation without delegating engine
  semantics.
- [x] Admit source and stylesheet bytes through a test-only bounded resource
  builder, release import handles, and seal an immutable snapshot; retain the
  public contract as unresolved.
- [ ] Compile one root template and one path/value expression.
- [ ] Execute `corpus/golden/hello` through batch-capable private machinery.
- [ ] Compare the semantic result separately from serialization.
- [ ] Add invalid, unsupported, missing/denied-resource, and budget cases with
  private structured diagnostics.
- [ ] Record navigation capabilities actually required by the slice for AR-0007.

Exit state: one exact golden result passes through named semantic owners with no
ambient I/O or public stability claim.

## Validation matrix

| Concern | Evidence | Required result |
| --- | --- | --- |
| Suite integrity | `check-conformance-sources.ps1` | Exact clean revisions |
| Catalog structure | `inventory-conformance-sources.ps1` | Retained counts; no missing/duplicates |
| Standards ownership | AR-0001 and resulting ADR | One named initial profile |
| Golden behavior | `corpus/golden/hello` | Exact classified result |
| Resource authority | Missing/denied and handle-release cases | No ambient fallback or retained handle |
| Diagnostics | Negative slice cases | Invalid and unsupported remain distinct |
| Repository gates | `scripts/verify.ps1` | Pass |

## Risks and mitigations

| Risk | Impact | Mitigation or evidence |
| --- | --- | --- |
| Catalog size is mistaken for implementation scope | Unbounded first milestone | Select dependencies only after AR-0001 |
| Private prototype leaks into public API | Premature compatibility burden | Keep facade empty until vertical evidence is reviewed |
| Harness defines semantics | Incorrect product ownership | Standards and FastXSLT tests remain authoritative over harness convenience |
| Tree representation spreads beyond XDM | Streaming/backend rewrite pressure | Record real navigation needs under AR-0007 |

## Progress log

### 2026-08-25

- Work completed: pushed scaffold checkpoint `f5e6064`; admitted and inventoried
  the pinned W3C suites.
- Validation: exact revisions, clean worktrees, 428/234 catalog references, no
  missing or duplicate sets, and 31,821/14,600 discovered cases.
- Findings: suite availability is adequate for modern-profile pressure but does
  not decide AR-0001 or cover the XSLT 1.0 alternative.
- Next slice: gather consumer transform families and complete candidate-profile
  suite evidence.

### 2026-08-25: Legacy-suite review and private-slice opening

- Work completed: reviewed the OASIS XSLT/XPath 1.0 Committee Draft 04 archive
  and recorded its catalog, doubts, acquisition, platform, and licensing facts.
- Findings: the archival suite is useful local evidence but unsuitable for
  automatic vendoring; current evidence favors a staged modern model.
- Plan change: opened Slice 2 only for private version-neutral ownership and
  resource-boundary work. Public standards semantics remain blocked.
- Next slice: implement bounded in-memory admission and immutable sealing for
  the existing golden source and stylesheet, including duplicate/size/budget
  failures and file-handle release evidence.

### 2026-08-25: Private bounded-resource experiment

- Work completed: added a test-only resource builder and immutable snapshot
  with opaque identity and explicit entry, per-entry-byte, and aggregate-byte
  limits.
- Validation: duplicate and empty identities, entry and byte limits, aggregate
  limits, equal bytes under distinct identities, and golden source/stylesheet
  rename/removal immediately after import.
- Findings: owned memory and handle release work without selecting public type
  names, URI interpretation, cache semantics, or batch behavior.
- Next slice: select the replaceable XML parser boundary for the private golden
  transform and record the exact semantic operations it must supply.

### 2026-08-25: Private XML parser experiment

- Work completed: opened AR-0008, compared three Rust parser candidates, and
  pinned `quick-xml` 0.40.1 only as a development dependency for the leading
  private experiment.
- Validation: in-memory parsing, expanded namespaces, resource identity and byte
  spans, malformed structure, duplicate expanded attributes, DTD and unknown
  entity denial, comment/PI retention pressure, and event/depth limits.
- Findings: pull events fit the owned-XDM seam, but FastXSLT must own document and
  namespace validation, external-authority policy, diagnostics, and limits.
  XML name/declaration/encoding conformance and production dependency admission
  remain open.
- Next slice: construct the first owned XDM document from the private adapter
  without retaining parser events or dependency-owned nodes.
