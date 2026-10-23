# ADR-0026: Evidence-Gated Profile Promotion and Frontier Progress

- Status: Accepted
- Opened: 2026-10-22
- Accepted: 2026-10-22
- Decision owner: Maintainer
- Review input: Monday's development-control notes supplied by the maintainer
- Related decisions: [ADR-0004](ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md), [ADR-0006](ADR-0006-verification-ledger-invariants.md), [ADR-0007](ADR-0007-staged-modern-standards-profile.md), [ADR-0023](ADR-0023-compilation-scoped-stylesheet-preprocessing-profiles.md), [ADR-0025](ADR-0025-compilation-graph-scoped-processor-edition-policy.md)
- Supersedes: None

## Context

A legitimate compatibility prerequisite can become an indefinitely expanding
private proof campaign. A corpus failure exposes an architectural seam; a review
selects an owner; a safe reference proves the direction; implementation then
keeps admitting neighboring constructs without a finite production exit plan.
More private tests pass while the observable product frontier remains unchanged.

The current [roadmap](../Plans/roadmap.md) illustrates this pressure: the
production OASIS baseline remains 2,504/3,173 exact comparisons (78.92%), while
selected-processor integration has accumulated substantial private evidence.
That work is useful, but private pass counts alone do not establish how much
closer the selected profile is to admission. These figures motivate this
decision; they are not thresholds or targets selected by it.

Existing decisions protect semantic ownership, safety and honest corpus
accounting. They do not yet require a finite campaign ledger connecting each
prerequisite tranche to an explicit promotion outcome. The missing boundary is
development control, not another XSLT semantic owner.

## Decision

**FastXSLT develops coherent, measurable, incrementally promotable standards
profiles—not exhaustive private completeness before production admission.**

**The purpose of the promotion ledger is to create an exit condition, not
another completeness checklist.**

Within the rollout scope below, a substantial compatibility or profile-integration campaign must declare
its target scope, finite promotion obligations and review point before further
scope expansion. Private evidence earns progress only through an identified
obligation, selected corpus frontier, or demonstrated architectural risk.

This decision governs sequencing and evidence. It does not lower any accepted
semantic, security, authority, diagnostic, accounting or validation requirement.

### 1. Separate measurement, promotion and release

- **Private measurement:** execute or classify the declared conserved denominator
  under an explicitly identified experimental profile. This can happen before
  production readiness. Unsupported and harness-limited cases remain visible.
- **Measured-profile promotion:** admit the declared, incomplete profile to the
  identified non-test engine path after its mandatory obligations and verification
  pass. The profile has a reproducible conserved report, not complete conformance.
- **Public release or default selection:** separately decide supported API,
  bindings, operational defaults, packaging and claims. Promotion alone grants
  none of these.

A report does not promote code. Moving code out of `cfg(test)` does not establish
support. Passing a private subset does not replace a full-denominator report.
An incomplete profile may be promoted only if its admitted behavior and explicit
unsupported boundaries are coherent and the applicable accepted gates are met.

### 2. Maintain a finite, versioned promotion ledger

The campaign plan must name:

- the processor semantics and preprocessing profile separately;
- the intended engine entry point/target and admitted semantic scope;
- the immutable corpus denominator and comparison/classification rules;
- applicable accepted decisions and their mandatory implementation gates;
- explicitly deferred features and non-goals;
- a finite list of promotion obligations and the next review boundary.

Each obligation has a stable ID, required invariant, affected owner/path,
objective exit criterion, reproducible validation command or test, evidence
reference, status, and next action. Use `open`, `in progress`, `closed`, or
`reopened`. Closure requires evidence for the stated scope, not a test-count
threshold or merely an implementation assertion.

Keep the ledger compact in the campaign plan. Detailed runs and completed
history belong in evidence documents. A gate may have a finite child checklist;
splitting it must preserve its identity and must not manufacture progress.

Once a promotion ledger is approved, its admitted semantic scope is closed for
that campaign revision. Newly implementable neighboring features are not
implicitly part of promotion. Expanding the admitted scope requires maintainer
review and a ledger revision; otherwise those features remain deferred even if
they are cheap to implement.

Adding or reopening a mandatory blocker requires recorded executable evidence
of a violated invariant, or an accepted requirement with a concrete failing or
missing validation witness, plus an explanation of why the declared profile
cannot coherently ship without it. Record the evidence immediately; changes to
the authoritative ledger follow the review rule below.
Missing code is not automatically a mandatory blocker: it may instead be an
explicitly unsupported feature outside the admitted scope.

Scope changes, additions/removal of mandatory obligations, and promotion require
maintainer review. A newly demonstrated correctness, security or accepted-contract
violation may be recorded immediately as a **provisional mandatory blocker** so
work does not proceed through a known defect. It changes the authoritative ledger
only at the next maintainer review. Until then, report it separately from the
existing mandatory denominator, for example, `9 mandatory + 1 provisional`.

The review must admit it as mandatory, classify it within an existing obligation,
defer it outside the admitted scope, or trigger a separate architectural decision.
Record the disposition, ledger revision and any count delta. A provisional issue
cannot be ignored to promote: it requires disposition first. A failed closure
test reopens the existing obligation immediately without increasing the
denominator. Deferring a defect never authorizes executing known-broken behavior
inside the approved scope. Conflicts with an accepted ADR return for a decision,
not a roadmap exception.

### 3. Report two progress tracks

At each completed tranche and promotion review, report:

1. **Corpus movement:** exact matches and other dispositions against the named
   denominator, separately for production and private profiles. Include the
   last full replay identity/date and distinguish newly measured cases from
   newly passing cases. A selected subset remains a subset.
2. **Promotion movement:** closed mandatory obligations out of the ledger's
   current total, IDs closed/reopened/added since the previous checkpoint,
   remaining blockers, separately listed provisional blockers, and the next
   closure action. Only reviewed additions change the mandatory denominator.

Gate counts are navigation aids, not effort percentages or release dates.
One remaining gate can cost more than several completed gates. A changed total
must show its reason; do not present a renumbered checklist as acceleration.
Private regression counts remain useful validation, but are neither corpus
credits nor evidence of promotion progress without a named obligation.

### 4. Apply a scope-expansion test

Before adding another private feature or experiment, identify at least one:

1. a mandatory promotion invariant it must establish;
2. an unchanged, already selected corpus case it is intended to unblock;
3. a concrete architectural correctness, security or resource-control risk;
4. a necessary distinction between admitted behavior and explicit unsupported
   behavior.

Record the expected result and stopping condition. If none applies, defer the
work to the ordinary backlog. Repeatedly choosing new corpus cases must not
evade the promotion review: a case-driven addition can be valid breadth work
without being a promotion prerequisite. Label it accordingly.

After a zero-gain tranche, record the blocker removed or risk resolved and
reassess the next action. Do not continue widening the same family solely
because the implementation now makes adjacent tests convenient.

### 5. Treat unsupported as a valid bounded outcome

A profile does not need every language feature before measurement or admission.
Unimplemented facilities may terminate in explicit unsupported classifications
with preserved provenance. They remain in the conserved denominator.

This is not permission to classify wrong supported behavior, internal crashes,
authority leaks, inconsistent error timing or dishonest introspection as
unsupported to close a gate. Recognition, implementation knowledge and actual
capability remain distinct under ADR-0025. A recognized but unimplemented
facility must not become standards-unknown or fallback-eligible for convenience.

Narrowing the declared scope requires an explicit review and a new report
identity where material; old evidence and original expectations remain intact.

### 6. Review after bounded semantic work

Default: review after **three semantic tranches**, or earlier when all
mandatory blockers are closed, a new architectural decision is required, or two
consecutive tranches close neither a promotion obligation nor a selected corpus
failure and resolve no named risk.

A tranche is one declared semantic or integration objective with its validation
and replay—not one tool call, chat turn, test function, commit or resumed agent
session. Carry the count across handoffs. A campaign may propose a different
finite interval with maintainer approval and a recorded reason; this is a
development checkpoint, not an engine or host resource limit.

Each review records exactly one disposition:

- **Promote:** obligations are closed, full conserved replay and required gates
  pass, and the maintainer approves the stated admission scope.
- **Continue:** name remaining blocker IDs, the next bounded work, and the next
  review point. More possible private tests is not a justification.
- **Defer:** preserve the tested reference, unresolved boundaries and a concrete
  reopening trigger; return effort to an independent frontier.
- **Abandon:** record the negative result and disposition the experimental code
  explicitly. Cleanup follows ordinary review and data-preservation rules.

No review outcome implicitly authorizes commits, deletion, publication or a
change to product defaults. A failed verification or incomplete replay prevents
promotion, even if all implementation items were previously marked closed.

### 7. Keep independent production breadth moving

Shared production fixes may proceed independently when they do not depend on
the unpromoted profile and satisfy their existing accepted contracts. A profile
campaign is not a blanket freeze on the production corpus frontier.

Conversely, a percentage target cannot justify bypassing required profile work,
fixture-specific behavior, per-case profile selection, relaxed comparison,
hidden exclusions or weakened controls. Architectural perfection outside the
declared scope is also not a reason to retain a coherent profile privately forever.

## Initial application and rollout

Apply this rule first to the current ADR-0025 processor-profile integration only;
do not retrofit historical ARs or create ledgers for every past campaign.
After the pilot's first promotion review, the maintainer will assess the process
and decide whether to make it the default for future substantial campaigns.

Before the next feature-expansion
tranche, create a concrete ledger mapping its nine accepted gate categories to
bounded closure obligations. Distinguish obligations already evidenced, actual
failures, missing validation and ordinary deferred feature breadth.

Do not invent a closed-gate count from the number of private probes. The current
qualification safeguard is a candidate closure item requiring completed review
and verification; this decision does not certify that implementation or close
the broader artifact-qualification gate.

Select the first review boundary and record both progress tracks in the roadmap.
Keep the current production and strict-preprocessing baselines distinct from a
future selected-processor full replay. Unsupported features need not be filled
in merely to run that replay.

## Alternatives considered

- **Percentage or time quotas:** pressure visible progress but incentivize
  unsafe shortcuts and penalize legitimate prerequisite work.
- **Unlimited private completeness:** produces strong local evidence without
  a bounded relationship to product admission.
- **Promote after enough tests:** test count says nothing about missing owners,
  untested failure paths or a conserved corpus denominator.
- **Freeze production breadth until profile completion:** unnecessarily blocks
  independent, standards-correct shared fixes.
- **Roadmap reminders alone:** useful sequencing, but too easy to replace with
  another attractive next case without a reviewable exit criterion.

## Consequences

Progress becomes observable even when the production numerator is stationary.
Campaigns can conclude with honest partial support or useful negative evidence.
Prerequisite work gains a bounded purpose without weakening existing safeguards.

The costs are ledger maintenance and periodic review. Gate-count gaming,
ever-growing subchecklists and ceremonial reviews are risks; stable identities,
explicit revisions and evidence-based closure are intended to counter them.

## Validation and acceptance checks

- Every mandatory obligation traces to an invariant and has a falsifiable exit
  criterion; mandatory work is not an unbounded feature wish list.
- A report shows both corpus and promotion deltas with unchanged accounting.
- New blockers remain provisional outside the authoritative denominator until
  reviewed; reopening an existing failed obligation does not inflate the total.
- Scope is closed between reviewed ledger revisions; neighboring features do
  not silently become prerequisites.
- Unsupported outcomes preserve standards distinctions and denominator membership.
- Review boundaries survive agent handoffs and produce an explicit disposition.
- Promotion retains the relevant ADR gates, full replay, target verification and
  maintainer approval; release/API/default decisions remain separate.

## Non-decisions

This decision selects no new processor semantics, public profile API, support claim,
default, release date, target percentage, scheduling implementation or ledger
storage schema. It does not accept any pending engine change or waive ADR-0025.

## Review history

- 2026-10-22 — Drafted from the maintainer's request and Monday's notes.
  Unnumbered and proposed; no change to binding contributor instructions.
- 2026-10-22 — Accepted with modifications in the supplied review: provisional
  treatment for new mandatory blockers until maintainer review, and closed
  campaign scope between reviewed ledger revisions. Three semantic tranches
  accepted as the default review interval; initial rollout limited to ADR-0025.
  Numbered ADR-0026. No engine implementation or profile promotion is accepted
  by this development-control decision.
