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
- The repository [Security Policy](../SECURITY.md) states the current threat
  scope, reporting route, and pre-stability security limitations.

## Review and active work

- [Architectural Reviews](Architectural%20Reviews/) preserve architectural
  questions, evidence, alternatives, findings, and dispositions. They do not
  override ADRs.
- [Plans](Plans/) describe executable work and sequencing. They do not change
  architecture by themselves.
- [Change Requests](Change%20Requests/) preserve needs from applications and
  sibling projects without making a consumer's implementation model part of
  the engine contract. Acceptance is not automatic.
- [Notes](Notes/) hold working observations and research that are useful but
  non-normative.

## Evidence

- [Evidence](Evidence/) records reproducible observations, audits, comparison
  results, and implementation checks.
- [Corpus](Corpus/) documents transform cases, external standards suites, and
  selection policy.
- [Safety and Limits](safety-and-limits.md) is the concise public summary of
  present support, trust, resource, and deployment limits. Specifications,
  ADRs, and the security policy remain authoritative where it summarizes them.

## Authority at a glance

| Location | Purpose | Authority |
| --- | --- | --- |
| `docs/Specifications/` | Current intended contracts | Normative within accepted ADRs |
| `docs/ADR/` | Accepted architectural decisions | Binding until superseded |
| `SECURITY.md` | Threat scope and vulnerability reporting | Repository policy |
| `docs/Architectural Reviews/` | Unresolved questions and evaluated evidence | Informative until disposition |
| `docs/Plans/` | Delivery slices and acceptance evidence | Execution guidance only |
| `docs/Change Requests/` | Consumer problems and requested boundaries | Incoming pressure, not a commitment |
| `docs/Evidence/` and `docs/Corpus/` | Reproducible observations and test inputs | Evidence, not guarantees |
| `docs/Notes/` | Working memory and research | Non-normative |

Public summaries such as this index and `safety-and-limits.md` make the current
state approachable. They must link to, rather than silently duplicate or
override, the owning contract.

## Placement guide

```text
Current intended contract?         -> docs/Specifications/
Accepted architectural decision?   -> docs/ADR/
Proposed ADR awaiting acceptance?  -> docs/ADR/Proposed/
Architecture under investigation?  -> docs/Architectural Reviews/
Executable implementation work?    -> docs/Plans/
Request from a consuming project?   -> docs/Change Requests/
Reproducible observation or audit?  -> docs/Evidence/
Working research or project memory? -> docs/Notes/
Test-suite or corpus policy?        -> docs/Corpus/
```

## Lifecycle

```text
Observation or consumer need -> Note, Evidence, or Change Request
                                      |
                                      v
                              Architectural Review
                                |-> defer / reject / no change
                                `-> Proposed ADR -> accepted ADR

Accepted ADR + specification -> Plan -> Code + tests + evidence
```

Not every change needs every document. Ordinary bug fixes and local refactors
that preserve accepted contracts do not require an AR or ADR.
