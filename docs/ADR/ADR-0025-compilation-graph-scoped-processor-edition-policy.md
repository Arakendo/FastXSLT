# ADR-0025: Compilation Graph Scoped Processor Edition Policy

- Status: Accepted
- Opened: 2026-10-16
- Accepted: 2026-10-16
- Related review: [AR-0031](../Architectural%20Reviews/AR-0031-compilation-scoped-processor-edition-profiles.md)
- Related decisions: [ADR-0007](ADR-0007-staged-modern-standards-profile.md), [ADR-0023](ADR-0023-compilation-scoped-stylesheet-preprocessing-profiles.md), [ADR-0006](ADR-0006-verification-ledger-invariants.md)
- Supersedes: None

## Context

OASIS fallback cases declaring version 2.0 expose a missing semantic owner:
the same declaration is a future version for an XSLT 1.0 processor but not for
the modern processor reference. A declaration cannot choose processor identity.
ADR-0023 selected stylesheet preprocessing only; extending its selector or a
local compatibility helper would conflate distinct decisions.

AR-0031's [bounded experiment and semantic audit](../Campaigns/processor-edition-profile/evidence/ar-0031-bounded-instruction-probe-2026-10-16.md#semantic-audit-and-decision-checkpoint)
contains 50 focused probes, whole-module lowering, bounded include/import
propagation, artifact isolation and qualification, and known-newer instruction
controls. The audit corrected modern missing-fallback error timing in the
reference rather than treating shared lowering as proof of semantic parity.
These observations justify an ownership boundary, not a completed profile.

## Decision

FastXSLT has one immutable **compilation-graph-scoped processor-edition policy**,
selected independently from stylesheet preprocessing and independently from
locally declared stylesheet versions. The policy governs edition-defined
recognition, forward/backward compatibility behavior, error timing,
availability/introspection and processor-version reporting through the shared
compiler and runtime.

This is an accepted behavioral ownership decision. It selects neither the
experiment's Rust representation nor production promotion. Production support
remains gated by this decision's executable validation requirements.

### Selection, scope and declared versions

The initial semantic directions are XSLT 1.0 processor semantics and the modern
reference defined by ADR-0007. Selecting a direction is not a conformance claim.
The selected processor interprets stylesheet version declarations; neither
principal nor local declarations select or switch the processor.

Principal stylesheets, transitive includes/imports, embedded stylesheets and
literal-result stylesheets inherit the same processor policy for a compilation.
Local version context retains the scope, lexical interpretation and effects
specified by that processor edition. It is not replaced by a graph-wide declared
version or a generic version-number interval. Compilation-graph scope does not
admit XSLT package facilities or a new dependency resolver.

Preprocessing remains a separate owner under ADR-0023. Initial coherent profile
integration pairs 1.0 processor semantics with strict-1.0 preprocessing, and
modern processor semantics with modern preprocessing. Orthogonal combinations
may remain private differential controls; they are not supported product
profiles. This decision freezes no public selector or default.

### Recognition, capability and introspection

Implementation knowledge, edition-defined recognition and implemented capability
are distinct. An instruction may be known to FastXSLT's implementation while
being unknown or unavailable under the selected processor edition. Conversely,
failure to implement a facility does not make it standards-unknown and therefore
fallback-eligible. Profile recognition must not depend on whether a lowering
branch happens to exist or succeeds.

Fallback eligibility and compatibility rules follow the selected edition and
effective declared-version context. Unsupported recognized facilities retain
an explicit unsupported disposition; they must not be disguised as unknown
instructions to gain a successful fallback result.

`element-available()`, `function-available()` and related introspection must
reconcile the selected edition with actual admitted capability and the applicable
standard. Recognition alone does not prove availability. Missing entries in an
experimental table do not establish a normative false result.

`system-property('xsl:version')` reports the selected semantic processor, not the
stylesheet declaration or maximum knowledge compiled into the binary. Its value
and atomic type must satisfy the selected edition and be truthful about the
implemented profile. The probe's strings are not selected reporting values or
proof of support. Until coherent reporting is implemented and evidenced, this
decision does not authorize advertising a complete processor edition.

### Shared execution and error timing

Both directions use the same compiler, XDM, lowering and runtime. Policy governs
which constructs are admitted and how compatibility affects their interpretation;
it must not create a second legacy evaluator or adapter-specific semantics.

Static versus deferred errors must follow edition-specific requirements at the
appropriate semantic boundary. A broad catch of compiler error codes is not a
substitute for those rules. Untaken branches, actual expression/function
evaluation, unknown instructions and missing fallback require independently
pinned behavior. Existing shared helpers may be reused only where their
eligibility and timing are proved for the selected policy.

Any deferred behavior must retain the immutable static context and original
provenance needed by shared execution, without capturing invocation state in
compiled artifacts. This decision does not change global-resolution scheduling,
runtime ownership, source views or result ownership under accepted ADRs.

### Artifact qualification and boundaries

Compiled modules and linked programs must retain sufficient immutable
qualification to prevent reuse or composition under a different processor or
preprocessing policy. Linking rejects incompatible artifacts rather than
relabeling them. Equal bytes, logical resource identities or coincident local
IDs do not establish compatible compilation semantics.

Any future reuse mechanism must include all relevant semantic qualification;
this decision creates no cache, registry, token layer or physical representation.
Qualification must survive include/import composition, failure cleanup,
concurrent compilation and compiled-artifact lifetime beyond input documents.

All host adapters project the same semantics. Edition selection grants no
filesystem/network authority, extension execution, DTD expansion, unsafe code,
or relaxed budgets. Preserve structured diagnostic categories, source locations,
work accounting and cancellation through both compilation and execution.

## Consequences

Processor identity has an explicit owner rather than emerging from unrelated
version helpers. Existing compatibility paths require migration and auditing;
the policy is not merely a new flag around fallback compilation.

Recognition and capability need a coherent account across lowering and
introspection. Artifact qualification adds obligations to every compilation and
composition path. Some archival outcomes may worsen when edition semantics
become consistent; that is not grounds for per-case policy selection.

Acceptance selects this direction only. Existing production defaults and
entry points remain unchanged until integration gates pass. No test-only wrapper,
four-module limit, diagnostic code, exact lexical witness set or version string
becomes a product contract.

## Alternatives considered

- Infer processor identity from principal/local version: circularly lets the
  stylesheet choose the processor that interprets it; fails the motivating seam.
- Reuse preprocessing as processor identity: exceeds ADR-0023 and prevents
  independent reasoning about whitespace and processor semantics.
- Patch fallback intervals alone: leaves recognition, expression timing and
  introspection inconsistent.
- Treat unimplemented facilities as unknown: silently turns engine limitations
  into standards-defined fallback eligibility.
- Separate legacy compiler/runtime: duplicates semantic ownership without
  evidence of necessity; the bounded shared-path experiment supports one engine.
- Keep collecting graph permutations before choosing an owner: no longer needed
  for this narrow decision; general graph behavior remains an integration gate.

## Validation and implementation gates

Before production profile integration:

1. Thread private immutable policy through every compilation path, preserving
   separate preprocessing and local-version context. Review responsibility seams
   under ADR-0004 rather than growing another monolithic compiler owner.
2. Pin edition-specific version lexical rules and local scope, including both
   attribute forms where applicable, ancestry, mixed-version include/import
   graphs, embedded and literal-result stylesheets. Extend beyond probe limits.
3. Test known-newer versus standards-unknown versus recognized-but-unimplemented
   facilities. Pin direct fallback and dormant/reached missing-fallback behavior
   under both editions using original source provenance.
4. Prove general expression static/deferred timing at actual evaluation/call
   boundaries, including dormant branches and errors that must remain static.
   Do not infer this from instruction-level catch-and-defer behavior.
5. Reconcile recognition, actual capability, availability functions and typed,
   truthful processor-version reporting. Probe tables and strings are not oracles
   for unimplemented entries.
6. Audit complete artifact qualification and linking, including imports,
   competing rules, failed recompilation, same-identity changed bytes, concurrent
   isolation and source-document release. Reject incompatible reuse explicitly.
7. Preserve budget boundaries, cancellation, diagnostics, authority and cleanup.
   Measure compilation and warm-execution costs independently of semantic wins.
8. Replay unchanged fallback corpus cases, strict/coherent 1.0 and modern lanes,
   and pinned modern regressions. Label processor and preprocessing separately;
   retain original expectations, denominators and explicit unsupported outcomes.
   Do not replace archival exact counts with corrected oracles or choose the
   better profile per case. No pass is earned by accepting this document.
9. Run repository verification and the official WASM target after code changes.
   Review public facade, supported-profile claims and defaults separately.

## Non-decisions

No complete XSLT 1.0 or modern conformance claim, public API/ABI/wire selector,
production default change, second backend, extension authority, cache, package
support or finalized feature catalog is selected. Profile integration remains
incremental and evidence-led; this decision does not promote experimental code.

## Review history

- 2026-10-16 -- Maintainer requested a proposed ADR after the 50-probe semantic
  checkpoint. Draft records the ownership direction and implementation gates;
  acceptance is pending.
- 2026-10-16 -- Maintainer accepted without modification: one immutable
  compilation-graph processor policy, distinct from preprocessing and locally
  declared versions, governs recognition, compatibility, error timing,
  introspection and artifact qualification through the shared compiler/runtime.
  Numbered ADR-0025; production support remains subject to the validation gates.
