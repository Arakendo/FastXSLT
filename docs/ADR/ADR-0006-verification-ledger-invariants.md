# ADR-0006: Verification Ledger Invariants

- Status: Accepted
- Date: 2026-08-25
- Related decisions: ADR-0002, ADR-0005
- Related reviews: AR-0001, AR-0011
- Supersedes: None

## Context

FastXSLT uses external standards suites and first-party fixtures as evidence.
The repository currently pins 31,821 QT3 cases and 14,600 XSLT30 cases, while
the W3C XML 20130923 archive remains a locally inventoried, non-admitted
candidate. A raw pass count cannot explain which cases were discovered,
selected, excluded, unsupported, interrupted, retried, or never executable by
the harness.

QT3, XSLT30, and XML have different catalogs, environments, dependencies,
assertions, and licensing boundaries. Flattening them into one normalized
corpus would obscure upstream meaning and risk turning the harness into a
second semantic engine. Conversely, reporting only cases that FastXSLT can
already execute would silently shrink the denominator.

Private experiments have retained suite-native identity across QT3 and XSLT30,
made unknown dependency and assertion metadata visible, and conserved
discovered and selected totals across filtering, sharding, interruption,
retry, and different merge orders. That evidence is sufficient to accept the
ledger invariants without choosing a public schema, storage engine, command
line, or conformance claim.

## Decision

FastXSLT verification infrastructure follows these invariants:

- Every discovered case has an explainable selection disposition. No case may
  disappear because it was inconvenient, unknown, unavailable, or not run.
- Selection/classification and execution/comparison are separate axes. Profile
  exclusion, engine capability, harness capability, invalid metadata,
  operation failure, semantic mismatch, and incomplete execution must not be
  conflated.
- Case identity includes the upstream suite, its immutable revision or digest,
  and the suite-native case identity. A filename, generated test name, shard,
  worker, or completion position is not sufficient identity.
- Suite-specific adapters retain native metadata and translate only the common
  facts needed by the ledger. They do not define FastXSLT language semantics,
  silently reinterpret unknown metadata, or modify upstream expected results.
- Unknown dependency, environment, or assertion metadata becomes a visible
  harness classification outcome. It is neither a pass nor a profile
  exclusion.
- Reports conserve discovered and selected denominators across filters,
  shards, retries, crashes, interruption, and order-independent merging.
  Duplicate or conflicting observations fail visibly.
- Report inputs include the corpus, profile/selection rules, overlays, engine,
  harness, target, toolchain, and relevant feature identity. A material input
  change creates a new report identity rather than rewriting old evidence.
- Conformance, adversarial, and performance corpora retain distinct purposes
  and reports. Evidence does not silently migrate between those purposes.
- Harnesses import explicitly selected fixture resources through bounded,
  host-controlled mechanisms. Corpus access does not grant ambient filesystem,
  network, entity, or output authority to engine execution.
- Parallel execution may change timing and completion order, but not case
  classification, environment isolation, assertion meaning, or result
  correlation.

This decision does not select the initial standards profile, a stable Rust or
serialized ledger schema, a storage format, CI tiers, assertion implementation
order, or wording for a published conformance claim. AR-0001 and focused
follow-on reviews retain those decisions.

## Consequences

- Progress reports remain honest when implementation or harness support is
  partial because every discovered case has a recorded disposition.
- Suite adapters can evolve independently while sharing conservation and
  identity rules.
- Retry and sharding machinery must retain enough identity to reject duplicate
  or conflicting observations.
- A smaller selected subset can be useful without being mistaken for the full
  suite or an unqualified conformance percentage.
- The harness carries additional accounting and provenance cost; that cost must
  be measured before selecting an all-in-memory or persistent representation.
- Published standards claims still require the accepted profile, exact suite
  revisions, applicable denominator, licensing/trademark review, and comparison
  coverage.

## Alternatives considered

### Generate one Rust test for every upstream case

Rejected as the architectural model because ordinary test-runner output does
not preserve selection reasoning, environment meaning, or denominator
conservation reliably.

### Normalize every suite into one first-party corpus

Rejected because normalization can lose native semantics and provenance,
expands licensing and maintenance surface, and encourages the harness to
duplicate engine behavior.

### Report only executable or passing cases

Rejected because it rewards silently narrowing the denominator and makes
unsupported or unknown cases disappear.

### Delay all corpus infrastructure until broad implementation

Rejected because early standards pressure is necessary to test engine
boundaries. The invariant layer can be accepted while schema and broad harness
implementation remain deferred.

## Verification

- Retain immutable revision and catalog-integrity checks for admitted suites.
- Test unknown metadata as a visible harness classification outcome.
- Test conservation under filtering, sharding, interruption, retry, and merge
  reordering.
- Reject duplicate inventory identity, observations outside the selected set,
  mixed report identity, and conflicting outcomes for one attempt.
- Keep first-party overlays separate from immutable upstream submodules.

## Reconsideration criteria

Reconsider or supersede this ADR if a suite cannot preserve its native identity
through the model, denominator conservation prevents practical execution at
required scale, a published standards process requires materially different
accounting, or a consumer requires a verification contract whose semantics
conflict with these invariants.
