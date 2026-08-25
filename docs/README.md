# FastXSLT Documentation

This directory separates current architectural truth, accepted decisions, open
questions, intended work, observed evidence, and non-normative project memory.
The location of a document signals its authority.

## Source of truth

- [Specifications](Specifications/) describe current intended contracts and
  boundaries within accepted ADRs.
- [ADRs](ADR/) record accepted, binding architectural decisions.
- [Testing Strategy](testing-strategy.md) defines the verification tiers and
  evidence required for compatibility or conformance claims.

## Review and active work

- [Architectural Reviews](Architectural%20Reviews/) preserve architectural
  questions, evidence, alternatives, findings, and dispositions. They do not
  override ADRs.
- [Plans](Plans/) describe executable work and sequencing. They do not change
  architecture by themselves.
- [Notes](Notes/) hold working observations and research that are useful but
  non-normative.

## Evidence

- [Evidence](Evidence/) records reproducible observations, audits, comparison
  results, and implementation checks.
- [Corpus](Corpus/) documents transform cases, external standards suites, and
  selection policy.

## Placement guide

```text
Current intended contract?         -> docs/Specifications/
Accepted architectural decision?   -> docs/ADR/
Proposed ADR awaiting acceptance?  -> docs/ADR/Proposed/
Architecture under investigation?  -> docs/Architectural Reviews/
Executable implementation work?    -> docs/Plans/
Reproducible observation or audit?  -> docs/Evidence/
Working research or project memory? -> docs/Notes/
Test-suite or corpus policy?        -> docs/Corpus/
```

## Lifecycle

```text
Observation -> Note or Evidence -> Architectural Review
                                     |-> defer / reject / no change
                                     `-> Proposed ADR -> accepted ADR

Accepted ADR + specification -> Plan -> Code + tests + evidence
```

Not every change needs every document. Ordinary bug fixes and local refactors
that preserve accepted contracts do not require an AR or ADR.

