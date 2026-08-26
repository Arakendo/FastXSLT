# AR-0011: Corpus Verification Ledger, Classification, and Reporting

| Field | Value |
| --- | --- |
| Status | Accepted |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | External and first-party test-data inventory, selection, execution, comparison, and evidence reporting |
| Trigger | FastXSLT has 46,421 admitted QT3/XSLT30 cases and 2,586 locally inventoried XML candidates, but only one upstream case executes through a first-party overlay |
| Related ADRs | ADR-0002, ADR-0005, ADR-0006 |
| Related reviews | AR-0001, AR-0004, AR-0006, AR-0008, AR-0010 |
| Related evidence | `docs/Evidence/w3c-suite-catalog-inventory-2026-08-25.md`, `docs/Evidence/xslt30-template-006-private-execution-2026-08-25.md`, `docs/Evidence/peer-test-corpus-review-monday-2026-08-25.md`, and `docs/Evidence/w3c-xml-conformance-suite-candidate-review-2026-08-25.md` |

## Architectural question

How should FastXSLT turn admitted or locally acquired test data into
reproducible verification evidence in which every discovered case has an
explainable disposition, without letting suite mechanics define engine
semantics, mixing different corpus purposes, hiding denominator changes, or
redistributing material without authority?

## Trigger and evidence

The repository pins 31,821 QT3 cases and 14,600 XSLT30 cases at immutable Git
revisions. Their catalogs are structurally inventoried, but they are not an
executable denominator. One overlay-selected XSLT30 case demonstrates that an
upstream case, environment, stylesheet, assertion, dependency, and revision can
be retained without copying fixture content.

The official W3C XML 20130923 archive adds 2,586 candidate cases. Its catalog
uses DTD/entity composition, its cases span multiple editions and processor
modes, and its redistribution boundary requires more review. It remains a
hash-identified local candidate rather than admitted corpus.

The current evidence therefore covers corpus discovery and one vertical case,
not a general classifier, assertion engine, execution ledger, reproducible
report, shard/retry model, or public conformance statement. AR-0001 has also not
selected the initial standards profile, so FastXSLT cannot yet define the
applicable denominator.

## Ownership and constraints

- AR-0001 and its eventual ADR own the selected XSLT, XPath, XML, XDM, and
  serialization profile. A harness does not infer the product contract from
  whichever cases are easiest to run.
- Corpus provenance owns canonical source, immutable revision/digest, license,
  acquisition, byte integrity, nested notices, and update procedure.
- A suite-specific inventory adapter owns decoding that suite's catalog,
  dependency, environment, and assertion metadata. It may not define FastXSLT
  language semantics or translate unknown metadata into silent exclusion.
- First-party overlays own deliberate selection, exclusion, expected
  unsupported classification, harness corrections, and issue references. They
  do not modify upstream fixtures or expected results.
- Engine layers own execution semantics and structured diagnostics. A harness
  consumes public or honestly private boundaries; it must not duplicate XPath,
  XSLT, XML, or XDM behavior to decide that the engine passed.
- Assertion comparators own comparison mechanics such as exact bytes, parsed
  XML, node/value sequences, error identity, serialization properties, or
  implementation-defined alternatives. Comparison type is explicit per case.
- A verification ledger owns case disposition, execution environment, engine
  revision, harness revision, and conservation totals. It is evidence, not a
  mutable engine cache or public runtime API.
- Conformance, adversarial, and performance corpora retain separate purposes and
  reports. A result in one family does not silently become evidence for another.
- ADR-0002 forbids a harness from granting engine execution ambient filesystem,
  network, entity, or output authority. The harness may import explicitly
  selected fixture bytes into bounded snapshots.
- ADR-0005 requires stable request/result identity independent of execution or
  completion order. Parallel harness execution cannot make list position a case
  identity or dependency mechanism.
- FastXSLT is MIT licensed. External corpus bytes and reports retain their own
  notices and usage restrictions and do not become MIT merely because the
  harness is MIT licensed.

## Candidate verification model

The leading model is a ledger with two related but distinct disposition axes.
Names below communicate required distinctions; they do not stabilize an enum,
schema, file format, or public API.

### Inventory and selection disposition

Every case discovered at a recorded corpus revision receives exactly one
selection disposition with a machine-readable reason:

- selected and applicable;
- excluded by the accepted standards/profile decision;
- known engine capability unsupported;
- harness capability unsupported;
- invalid, contradictory, or unresolved suite metadata; or
- unavailable because the corpus or a required resource was not admitted.

Unknown dependencies and assertions fail classification visibly. They are not
treated as profile exclusions.

### Execution and comparison disposition

Every selected case receives exactly one execution disposition:

- comparison passed;
- semantic result mismatch;
- expected diagnostic or operation-failure mismatch;
- unexpected engine operation failure or internal failure;
- harness execution/comparison failure; or
- not completed under an explicitly recorded interrupted run.

An expected standards error that matches its assertion is a passed comparison,
not an engine operation failure. Cancellation, resource exhaustion, worker
failure, and harness transport failure remain distinct from standards-defined
negative cases under AR-0004 and AR-0010.

### Conservation and reproducibility

A report must make denominator conservation mechanically checkable. At minimum:

```text
discovered cases
    = selected
    + excluded by profile
    + engine unsupported
    + harness unsupported
    + metadata/resource classification failures

selected
    = passed
    + semantic/diagnostic mismatches
    + engine operation failures
    + harness execution failures
    + explicitly incomplete cases
```

The exact vocabulary may evolve, but no discovered or selected case disappears.
The ledger records corpus identity, immutable revision/digest, case identity,
selection rule/reason, dependencies, environment, assertion kind, engine and
harness revisions, target/toolchain/features, and execution/comparison outcome.

Reports are derived artifacts. Re-running from the same inputs should reproduce
case membership and dispositions, while timing and scheduling observations may
vary. A changed suite revision, profile, overlay, harness, engine, feature set,
or environment creates a new report identity rather than mutating historical
evidence.

## Alternatives

### A. Directly turn every upstream case into a Rust test

This gives familiar test-runner output but hides selection logic in generated
test names, makes missing cases hard to detect, encourages fixture I/O during
execution, and poorly represents suite environments, optional assertions,
unsupported dependencies, and report conservation.

### B. Build one universal normalized corpus and harness

Copy every suite into one FastXSLT schema and execution engine. This could make
querying uniform, but normalization risks losing upstream identity and meaning,
creates a large maintenance and licensing surface, and invites the universal
harness to become a second semantic implementation.

### C. Use suite-specific adapters feeding a shared verification ledger

Each adapter retains its upstream metadata and translates only into a small
shared inventory/disposition/reporting model. Execution and comparison reuse
FastXSLT boundaries and focused assertion owners. This is the leading direction
because it permits shared conservation/reporting without pretending QT3,
XSLT30, and XML catalogs are the same animal.

### D. Report only cases the engine currently supports

This shortens reports and accelerates early implementation, but destroys the
denominator, hides unsupported growth, and makes progress appear better when
selection narrows. It is rejected as a verification model.

### E. Defer all corpus infrastructure until the full profile is implemented

This avoids premature abstraction, but implementation would proceed without
standards pressure and later harness work could expose foundational semantic
mistakes. Inventory and one-case vertical adapters can proceed privately before
the profile closes; generalized execution remains deferred.

## Findings and uncertainties

- Every inventoried case needs an explainable disposition; “not run” requires a
  reason and cannot be an omitted row.
- Selection and execution are separate axes. A case can be applicable but
  unsupported, or selected and fail in the harness before engine execution.
- Suite-specific adapters are necessary because QT3, XSLT30, and XML metadata,
  environments, assertion vocabularies, and licensing/acquisition differ.
- A small shared ledger/reporting core is plausible, but one executed upstream
  case is insufficient evidence to choose its Rust types, persistence format,
  database, generated-test strategy, or command-line interface.
- Case identity must include upstream suite plus immutable revision and native
  case identity. Filename or generated Rust test name is not enough.
- Report totals must conserve both discovered and selected denominators across
  sharding, filtering, retries, crashes, and interrupted runs.
- Parallel execution may reorder completion but cannot change classification,
  environment isolation, assertion meaning, or result association.
- Published conformance language and W3C trademark/license use require a
  separate review of the exact unmodified suite, subset, and report. An internal
  development ledger is not automatically an authoritative conformance claim.
- It remains unknown which assertion families should be implemented first, how
  suite environments map to sealed snapshots, how expensive full ledgers are,
  and whether reports should be JSON, SQLite, another format, or ephemeral plus
  signed/hashed artifacts.
- A private two-suite experiment now retains QT3 `assert-eq` and XSLT30
  compound message-assertion metadata in suite-specific records, then projects
  only identity plus separate selection/execution dispositions into a common
  record. This is useful boundary evidence, not a stable schema. Unknown
  assertion and dependency metadata becomes a visible harness-unsupported
  outcome.
- A separate private accounting experiment conserves discovered and selected
  totals across profile filtering, two shards, an interrupted case, and a
  successful retry. Merge order does not affect the result; conflicting
  same-attempt outcomes, mixed run/shard identity, duplicate inventory cases,
  and execution observations outside the selected set fail visibly. Attempt
  ordinals remain experimental mechanics rather than an adopted schema.

## Disposition

**Accepted through ADR-0006.** Verification must preserve native case identity,
classify every discovered case, separate selection from execution, conserve
denominators, retain report inputs, and keep corpus purposes distinct.

ADR-0006 deliberately does not stabilize a universal harness, ledger schema,
report database/API, CI topology, or published conformance percentage. Those
follow-on choices remain deferred until AR-0001 closes and executable standards
slices provide the relevant comparison and scale evidence.

## Required follow-up

- [x] Reproducibly inventory QT3 and XSLT30 root catalogs at immutable
  revisions, retaining zero missing or duplicate root references.
- [x] Execute one XSLT30 case through an explicit overlay while retaining native
  case, environment, stylesheet, assertion, dependency, and revision identity.
- [x] Inventory the W3C XML archive candidate and identify its secure catalog,
  selection, acquisition, and rights constraints without admitting it.
- [ ] Close AR-0001 with an accepted profile that can classify applicability.
- [x] Define a private case-record experiment using at least one QT3 assertion
  family and a second XSLT30 assertion/environment family.
- [x] Demonstrate conservation totals when a run is filtered, sharded,
  interrupted, retried, and merged in different completion orders.
- [x] Demonstrate that unknown dependency/assertion metadata becomes a visible
  harness classification failure rather than exclusion or pass.
- [ ] Map selected suite environments into bounded sealed snapshots without
  ambient path/network/entity access or retained file handles.
- [ ] Establish comparison ownership for exact text, parsed XML, sequences,
  serialization assertions, expected standards errors, and permitted
  alternatives as each becomes executable.
- [ ] Define immutable report identity and provenance fields before retaining or
  publishing generated reports.
- [ ] Decide local-only versus distributable XML-suite acquisition after a
  focused rights review.
- [ ] Measure ledger/classification memory and runtime cost before selecting a
  storage format or loading all cases into memory.
- [ ] Define CI tiers so focused pull-request evidence remains fast while full
  corpus runs are reproducible and cannot silently shrink.
- [ ] Review published-report wording, W3C licensing/trademark obligations, and
  subset rules before making any conformance or standards-performance claim.

## Reopening triggers

Revisit or supersede this review when AR-0001 accepts a standards profile, a
second assertion family executes, XML candidate rights are resolved, report
volume requires persistent storage, sharding loses or duplicates cases, a
consumer needs machine-readable verification results, or published conformance
reporting becomes a release requirement.

## Review history

- 2026-08-25 -- Opened as Incubating after corpus review established 46,421
  admitted QT3/XSLT30 cases plus 2,586 non-admitted XML candidates and identified
  explainable disposition—not raw pass count—as the verification objective.
- 2026-08-25 -- Accepted the evidence-backed identity, classification,
  conservation, provenance, and corpus-purpose invariants through ADR-0006;
  retained schema, storage, CI, comparison, and publication questions as
  deferred follow-up.
