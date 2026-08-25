# ADR-0004: Source Unit Cohesion, Size Pressure, and Decomposition

- Status: Accepted
- Date: 2026-08-25
- Related decisions: ADR-0001, ADR-0002, ADR-0003
- Source precedent: Tokimu ADR-0015, reframed for FastXSLT
- Supersedes: None

## Context

FastXSLT will accumulate exacting language semantics, conformance regressions,
diagnostic provenance, resource-policy checks, performance experiments, host
adapters, and possibly multiple execution strategies. Successful test and
corpus work naturally increases implementation and evidence. Without a review
rule, that evidence can concentrate unrelated responsibilities in one source
unit until ordinary changes become difficult to navigate, attribute, test, and
review safely.

Likely pressure points include:

- XML adaptation and serialization;
- XDM node storage, identity, navigation, values, and sequences;
- XPath lexing, parsing, static analysis, evaluation, and function families;
- stylesheet structure, static resolution, semantic normalization, lowering,
  template dispatch, imports, modes, keys, variables, and functions;
- resource snapshots, URI resolution, caches, and batch/graph execution;
- structured diagnostics and retained source provenance;
- ASP.NET/FFI ownership, ABI, cancellation, and result transfer;
- conformance selection, expected-result comparison, and reporting; and
- benchmark, fuzz, differential, and regression harnesses.

Line count alone is not an architectural defect. A short file can mix several
owners, while generated bindings, exact test catalogs, or cohesive declarative
tables can be legitimately large. Pressure becomes architectural when a source
unit no longer communicates one coherent implementation responsibility.

> File size triggers review; responsibility boundaries justify decomposition.

> Split by meaning, not by line count.

## Decision

FastXSLT hand-maintained source units represent one coherent implementation
responsibility. Size and responsibility signals trigger an explicit cohesion
review. The review may retain a cohesive unit, require behavior-preserving
decomposition, or record a bounded exception.

This policy applies proportionally to Rust, C#, TypeScript/JavaScript,
PowerShell, build logic, test harnesses, and other hand-maintained implementation
source. It does not make filesystem layout an architectural owner, force crate
creation, or move host/adapter behavior into the engine core.

### Size review triggers

Physical line count is a deliberately inexpensive first signal:

| Hand-maintained source size | Required treatment |
| --- | --- |
| Up to 1,000 lines | Ordinary. No size justification required; cohesion may still require review. |
| 1,001–2,000 lines | Inspect cohesion during a substantive modification. |
| 2,001–4,000 lines | Perform and retain an explicit decomposition review. |
| More than 4,000 lines | Presume decomposition debt; retain only with a documented cohesion or sequencing reason. |
| More than 8,000 lines | Exceptional; active work should normally include a checkpointed decomposition campaign. |

Comments, embedded tests, and hand-maintained lookup data count because they
contribute to the same navigation and review burden. These thresholds are review
signals, not quality scores, merge failures, or automatic commands to create
more files. CI may report crossings but must not fail solely on physical line
count unless a later decision demonstrates value for a mechanical gate.

### Responsibility review triggers

A cohesion review is required regardless of size when a unit exhibits two or
more of these conditions:

- it contains multiple independently testable language phases, semantic
  subsystems, or host boundaries;
- it mixes stable reference semantics with experimental or optimized paths;
- it mixes XML/XDM/XPath/XSLT meaning with CLI, ASP.NET, FFI, filesystem,
  presentation, or platform mechanics;
- it combines resource authority, cache policy, batch scheduling, and language
  evaluation without a clear composition owner;
- it contains several unrelated diagnostic, tracing, comparison, or reporting
  systems;
- tests require navigating substantial unrelated implementation;
- ordinary changes routinely touch distant unrelated regions;
- generic, macro, monomorphization, or code-generation coupling makes a unit a
  disproportionate incremental-build or compile-time bottleneck;
- its filename no longer communicates useful subject ownership and
  responsibility;
- a coherent responsibility can move behind a private module without changing
  a stable contract; or
- multiple ADRs, Architectural Reviews, standards slices, or conformance
  campaigns independently modify the same unit.

Crossing a size threshold and satisfying one responsibility trigger is also
sufficient to require review.

### Decomposition follows ownership seams

Every extracted unit names a responsibility. Decomposition must not create
numbered fragments, arbitrary line buckets, generic `helpers`, `misc`, or
`utils` dumping grounds, or a directory whose files only make sense as one
anonymous continuous file.

Potential FastXSLT seams include syntax, static context, semantic normalization,
evaluation, template selection, resource adaptation, result construction,
serialization, diagnostics, comparison, and host interop. These are illustrative
responsibilities, not preaccepted modules or public APIs. Actual code, callers,
and evidence decide which seams exist.

Prefer the smallest sufficient visibility:

1. private child module in the same crate or host adapter;
2. crate-visible module needed by several local units;
3. an existing public abstraction whose accepted responsibility already fits;
4. a new public API, crate, native ABI, or package only after callers and an
   Architectural Review justify the stable boundary.

ADR-0001 remains controlling: making a file smaller does not justify splitting
the modular monolith into crates. Moving code does not transfer semantic
ownership. Filesystem ownership and architectural ownership are distinct.

### Subject and responsibility identify a unit

Directory structure communicates the owned subject. A filename communicates
the implementation responsibility within that subject. Together they should
answer “which meaning does this belong to?” and “what work does it perform?”

For example, a demonstrated future layout might resemble:

```text
xpath/
    parse/
        mod.rs
        expressions.rs
        sequence_types.rs
        diagnostics.rs
    functions/
        strings.rs
        numerics.rs
        nodes.rs

xslt/
    compile/
        mod.rs
        declarations.rs
        template_rules.rs
        imports.rs

host/
    dotnet/
        abi.rs
        ownership.rs
        cancellation.rs
        diagnostics.rs
```

This example does not admit those exact modules, public names, crate boundaries,
or even a `host` directory. Human-readable semantic names are required; empty
ceremonial files and opaque abbreviations are not.

Before accepting an extracted unit as coherent, the review can state:

- its subject and implementation responsibility;
- the semantics, authority, policy, or state it owns;
- its inputs and dependencies;
- its outputs, effects, or observations;
- its permitted dependency direction; and
- the responsibilities it explicitly must not own.

If those answers are unclear, keep the candidate private and local while the
boundary is studied. Uncertainty is not evidence for a public interface.

### Successful extraction reduces responsibility coupling

Smaller files do not prove successful decomposition. New modules must not
recreate the monolith through conceptual cycles, unrestricted sibling-internal
access, parent pass-through methods that disguise cycles, or a broad mutable
context containing nearly every former responsibility.

After extraction, inspect:

- dependency direction among the units;
- state and policy each reads or mutates;
- whether the unit can be tested through its named responsibility;
- whether most units still require most of the former unit's context;
- whether shared types represent a real lower-level contract or merely moved
  coupling; and
- whether hot-path calls now pay indirection, allocation, cloning, dispatch, or
  host-boundary costs solely to satisfy the file layout.
- whether the change improves, preserves, or worsens incremental and clean build
  time, including downstream recompilation and monomorphization pressure.

Some coordination through a composition root is expected. The trial is
unsuccessful when its modules remain mutually dependent on nearly all former
state or policy. Record and revise that result rather than calling physical
fragmentation architectural progress.

> A successful decomposition reduces responsibility coupling, not merely
> physical source size.

### Tests are retained and organized by invariant

Regression growth is evidence that the engine is learning. It does not require
every regression to remain beside the production code that first exposed it.

Private implementation tests may live with private modules. Contract,
conformance, golden, differential, host-integration, safety, and composition
tests should use the narrowest honest boundary. Organize tests and fixtures by
retained invariant—for example node identity/order, namespace resolution,
effective boolean value, template priority, import precedence, snapshot
immutability, batch isolation, diagnostic provenance, or ABI ownership—rather
than by an arbitrary source split.

Moving tests must not weaken exact assertions into serialization-only snapshots,
silently change suite selection, duplicate engine semantics in helpers, or erase
unsupported/failure classifications.

### Active semantic work uses checkpointed decomposition

A threshold crossing does not authorize an unrelated refactor in the middle of
an unresolved standards, conformance, performance, security, or unsafe-code
investigation. For an actively changing exceptional unit:

1. reach and document a coherent checkpoint;
2. retain passing tests, failing/unsupported cases, suite manifests, benchmark
   baselines, diagnostics, fingerprints, and known falsifications;
3. perform behavior-preserving extraction separately from semantic repair or
   optimization;
4. rerun the same evidence gates after each coherent extraction group; and
5. resume semantic work only after the new composition reproduces the
   checkpoint.

If no safe checkpoint exists, record why extraction would increase attribution
risk and name the next bounded checkpoint.

### Conservation requirements

A decomposition must preserve every applicable contract:

- architectural ownership and dependency direction;
- public Rust APIs, native ABI, managed API, and visibility unless another
  accepted decision changes them;
- XDM node identity, document order, values, and lifetime behavior;
- XPath/XSLT static and dynamic semantics, evaluation order where observable,
  template priority, import precedence, result association, and error behavior;
- snapshot identity, admitted authority, limits, cache visibility, batch order,
  cancellation, and concurrent isolation;
- transformation result meaning and serialization behavior;
- structured diagnostic identities, primary/related locations, and source
  provenance;
- conformance selection, unsupported/failure classifications, and expected
  results;
- performance baselines whose contracts have not intentionally changed; and
- all accepted unsafe-code safety contracts and exact unsafe surface under
  ADR-0003.

A mechanical extraction must not quietly repair, suppress, or reinterpret an
active defect. Record a semantic fix discovered during extraction and perform it
as a separate reviewable change after restoring the conservation baseline.

### Unsafe and FFI boundaries

Decomposition never authorizes unsafe code. If a later ADR admits a narrow
unsafe exception, extraction must not spread its invariants across more modules,
increase its unchecked surface, or move validation away from the safe boundary
without reopening that exception.

FFI decomposition preserves ABI layout/version, allocation ownership,
nullability, encoding, callback lifetime/threading, panic containment,
cancellation, and error transfer. A smaller `abi.rs` is not an improvement if
the safety contract becomes distributed and harder to audit.

### Exceptions

Graduated thresholds do not apply directly to:

- generated code or machine-produced bindings;
- vendored source;
- exact W3C or other external corpus artifacts;
- generated conformance catalogs and result manifests;
- static lookup tables or data-dominant units;
- cohesive declarative grammar/schema sources; or
- normative documentation whose comprehension depends on one continuous
  decision or specification record.

An exception identifies its category, separates generated/data content from
hand-maintained logic where practical, records generation and provenance, and
explains why the unit is more coherent intact. “Splitting is inconvenient” is
not sufficient.

Generated output does not exempt a large hand-maintained generator. Vendored or
generated content must not be edited manually merely to satisfy a size report.

### Review record

An explicit cohesion/decomposition review records:

- current line count and responsibility inventory;
- thresholds and triggers that caused review;
- proposed subjects, responsibilities, ownership, inputs, outputs, exclusions,
  and dependency direction;
- expected reduction in responsibility coupling;
- public, ABI, dependency, safety, memory, and hot-path consequences;
- measured or observed clean-build, incremental-build, and downstream
  recompilation consequences where compile-time pressure triggered the review;
- checkpoint and conservation evidence;
- disposition: decompose, retain with reason, or defer to a named checkpoint;
  and
- reopening triggers.

A multi-step campaign belongs in a plan. It distinguishes behavior-preserving
moves from subsequent fixes, optimizations, and API changes.

## Initial application

No current FastXSLT implementation unit crosses the numeric thresholds. The
project is at scaffold stage, so this ADR prevents pressure rather than
retrofitting an existing monolith.

The first mandatory review will occur when a hand-maintained unit crosses the
2,000-line threshold, exceeds 1,000 lines while satisfying a responsibility
trigger, or satisfies two responsibility triggers at any size. That review is
the initial calibration pilot and may revise the thresholds if FastXSLT's real
language/compiler code demonstrates materially different pressure.

This ADR does not require ceremonial pre-splitting of empty modules or immediate
automation. Early vertical slices may remain concrete and local until actual
responsibilities become visible.

## Alternatives considered

### No shared rule

Rely on local judgment. This avoids process but allows successful conformance
and performance work to accumulate unrelated responsibilities without a named
review point.

### Hard maximum line count

Fail CI whenever a source file exceeds a fixed size. This mistakes size for
cohesion, encourages arbitrary fragmentation, and mishandles generated,
vendored, corpus, and declarative content.

### Tooling report without a decision rule

Report source-size statistics but give crossings no meaning. This locates
pressure without deciding whether to retain, decompose, or conserve behavior.

### Graduated cohesion review

Use size as a cheap signal, responsibility as the justification, private
extraction as the default structural response, and checkpointed evidence to
preserve semantics and attribution.

## Consequences

### Positive

- Large or contested units receive review before change isolation degrades.
- Conformance and regression evidence can grow without forcing all logic into
  one production file.
- Private decomposition improves navigation without manufacturing public APIs,
  crates, ABIs, or alternate semantic owners.
- Checkpoints distinguish structural movement from language repair and
  optimization.
- Legitimately large generated, corpus, data, grammar, and normative units have
  bounded exceptions.

### Negative

- Reviews and conservation reruns add work.
- Extraction may expose coupling that requires careful sequencing.
- A cohesive unit may temporarily cross a threshold and need a written retain
  disposition.
- Numeric signals can be misread as quality scores unless reviewers continue to
  prioritize responsibility.

## Reopening triggers

Revisit this decision when:

- reviews become ritual justifications with no useful outcome;
- thresholds cause arbitrary fragmentation or disproportionate churn;
- compiler/evaluator hot paths materially regress because responsibility seams
  introduced avoidable costs;
- compile-time observations repeatedly justify a different trigger or show that
  decomposition merely relocates monomorphization and downstream rebuild cost;
- generated, corpus, grammar, or data-heavy units remain misclassified;
- private extraction repeatedly forces accidental public APIs or crates;
- conservation cannot reliably retain standards, diagnostics, snapshot, batch,
  ASP.NET/FFI, performance, or safety evidence; or
- the first calibration pilot demonstrates better thresholds or triggers.
