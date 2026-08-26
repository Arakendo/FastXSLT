# ADR-0007: Staged Modern Standards Profile

- Status: Accepted
- Date: 2026-08-26
- Related decisions: ADR-0002, ADR-0006
- Related reviews: AR-0001, AR-0007, AR-0008, AR-0011
- Supersedes: None

## Context

FastXSLT needs a named semantic direction before implemented behavior can grow
into a public contract. Starting with XSLT 1.0 would reduce the immediate
surface, but risks making node-set and value behavior the engine's permanent
foundation. Claiming XSLT 3.0 conformance from the start would instead turn a
small implementation into a misleading product claim.

The repository pins the W3C QT3 and XSLT 3.0 suites. Their native catalogs
provide modern dependencies, environments, assertions, and expected results.
The archival XSLT 1.0 suite reviewed by the project remains useful local
compatibility evidence, but its age and redistribution constraints make it a
poor primary public denominator.

The complete six-case XSLT30 `template` test set now demonstrates the intended
accounting model: one case passes and five remain explicitly engine-unsupported.
This is enough to choose a semantic direction and a widening discipline. It is
not evidence of broad conformance or consumer workload fitness.

## Decision

FastXSLT adopts a **staged modern standards profile**. The semantic reference
editions are:

- [XSL Transformations (XSLT) Version 3.0](https://www.w3.org/TR/xslt-30/);
- [XML Path Language (XPath) 3.1](https://www.w3.org/TR/xpath-31/);
- [XQuery and XPath Data Model 3.1](https://www.w3.org/TR/xpath-datamodel-31/);
- [XSLT and XQuery Serialization 3.1](https://www.w3.org/TR/xslt-xquery-serialization-31/);
- [Extensible Markup Language (XML) 1.0, Fifth Edition](https://www.w3.org/TR/xml/);
  and
- [Namespaces in XML 1.0, Third Edition](https://www.w3.org/TR/xml-names/).

These editions define the direction and meaning of implemented features. They
do **not** create a claim of XSLT 3.0 basic conformance, complete XPath 3.1,
complete XDM 3.1, complete serialization support, or support for every XML
encoding and optional processor feature.

The preview is feature-enumerated and incomplete. A feature becomes supported
only when its owned semantics, diagnostics, resource and work limits, and
representative comparison evidence are implemented. Syntax shared with XSLT
1.0 or 2.0 does not make FastXSLT a conforming processor for those editions;
legacy compatibility remains evidence to acquire and a product choice to make.

The pinned QT3 and XSLT30 revisions recorded in the corpus provenance document
are the primary standards corpora. First-party overlays own selection,
exclusion, expected capability, and harness corrections without modifying
upstream data. ADR-0006 governs case identity, classification, and denominator
conservation.

The first preview denominator is the complete six-case XSLT30 `template` test
set at the pinned suite revision. It is an accounting and implementation
baseline, not the definition of the supported profile. Its current disposition
is one selected pass and five explicit engine-unsupported cases.

Scope widens by coherent semantic family. Before a family is described as
supported, FastXSLT must:

1. name the standard behavior and the complete upstream metadata selection that
   exercises it;
2. implement the behavior through the engine-owned semantic layers rather than
   delegate it accidentally to a parser, host, or comparison harness;
3. preserve structured unsupported, invalid, authority, budget, and internal
   outcomes where relevant;
4. retain every discovered case in the ledger, including failures and
   unsupported or harness-unsupported outcomes; and
5. update the feature description and evidence without implying support for
   adjacent unimplemented features.

The initial profile deliberately excludes claims for schema awareness, XSLT
streaming, packages, extension functions, dynamic evaluation, and ambient
filesystem or network resolution. These capabilities require focused evidence
and architectural review. XML decoding/encoding admission, parser production
admission, and the exact serialization method set remain owned by their focused
reviews and implementation evidence.

Representative consumer transforms remain required before FastXSLT claims
application fitness, prioritizes optional compatibility around a named
customer, stabilizes host lifecycle defaults, or publishes ASP.NET performance
claims. They are not a prerequisite for standards-directed implementation.

## Consequences

- XDM and XPath may grow toward modern sequence, name, type, and context
  semantics instead of treating a 1.0 node-set model as the permanent core.
- Roadmap work can select complete QT3/XSLT30 metadata families while reporting
  partial implementation honestly.
- A passing XSLT30 case is evidence for that case only; unsupported cases remain
  in the denominator.
- Compatibility with XSLT 1.0 and 2.0 requires explicit evidence and cannot be
  inferred from shared syntax or a stylesheet's `version` attribute.
- XML 1.0 and Namespaces 1.0 semantic editions are selected, while AR-0008
  still owns parser admission, encodings, DTD/entity policy, and corpus rights.
- The size of the modern standards surface increases implementation work, but
  does not require implementing unrelated features before a useful preview.

## Alternatives considered

### XSLT 1.0 and XPath 1.0 first

Rejected as the primary semantic foundation. It may still become a named
compatibility profile, but the available primary corpus and desired growth path
favor modern XDM/XPath semantics.

### Claim XSLT 3.0 basic conformance immediately

Rejected because the current implementation and evidence do not approach that
claim. Naming reference editions must not turn aspiration into support.

### Remain version-neutral

Rejected because names, values, sequences, patterns, diagnostics, test
applicability, and widening order would otherwise be settled by private
implementation accidents.

## Verification

- Keep QT3 and XSLT30 immutable at the revisions in the corpus provenance
  record and fail verification on a silent gitlink move or dirty submodule.
- Retain the complete `template` test-set denominator and its native metadata.
- Add complete, metadata-driven case families as semantics widen; never select
  only already-passing cases to manufacture a favorable denominator.
- Record exact standards, suite, engine, harness, overlay, toolchain, and target
  identity before publishing a report.
- Keep conformance, adversarial, and performance evidence distinct.

## Reconsideration criteria

Reconsider or supersede this ADR if representative consumers require a
conflicting compatibility baseline, modern data-model choices prevent an
important legacy profile, the pinned suites cease to be suitable primary
evidence, or FastXSLT seeks a formal conformance designation with requirements
that materially change this staged policy.
