# AR-0013: Prepared Representation and Data-Layout Audit

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-27 |
| Last reviewed | 2026-08-27 |
| Scope | XDM, compiled stylesheet, execution plan, prepared input, and invocation-local storage |
| Trigger | Explore whether deliberately prepared representations can improve repeated execution rather than inheriting conventional engine layouts without evidence |
| Related ADRs | ADR-0002, ADR-0003, ADR-0004, ADR-0007 |
| Related reviews | AR-0007, AR-0009, AR-0012 |
| Related evidence | `../Evidence/private-prepared-input-reuse-2026-08-25.md`; `../Evidence/private-prepared-retention-observation-2026-08-25.md`; `../Evidence/allocation-counter-review-and-preparation-probe-2026-08-25.md`; `../Evidence/aspnet-native-vs-isolated-tiered-comparison-2026-08-26.md` |

## Architectural question

Which private data representations, indexes, execution-plan specializations,
reusable scratch layouts, and Rust-level implementation techniques—if any—materially improve correct repeated
transformation under FastXSLT's bounded memory and host-neutral lifecycle,
without stabilizing implementation details or making preparation more expensive
than the execution it is intended to accelerate?

## Trigger and evidence

FastXSLT already benefits from compiling stylesheets once and reusing immutable
prepared XDM state. Existing probes also show the cost: preparation retains more
memory, and the relative value depends on source shape, transform work, reuse
count, concurrency, and the host boundary. The current implementation was built
to establish semantics and lifecycle evidence. Its Rust containers and layouts
must not become permanent merely because they were convenient for the reference
path.

There is no current profile showing that a particular arena, index, interner,
sequence representation, or cache is the next bottleneck. There is also no
evidence that a novel representation will outperform straightforward safe Rust
after preparation cost, retained memory, diagnostics, cancellation, and result
transfer are included. This review preserves the investigation without
presuming that novelty is valuable.

## Ownership and constraints

- XDM owns node identity, document order, names, values, and navigation
  semantics. A physical representation may optimize them but may not redefine
  them.
- Stylesheet compilation owns stylesheet-derived static knowledge and may use
  it to select or specialize an execution plan. Source documents, parameters,
  resolver state, clocks, messages, and budgets remain invocation or prepared-
  input concerns under their existing contracts.
- Prepared state remains immutable and source-derived under AR-0009. Content
  equality must not collapse document identity, and no global or cross-snapshot
  cache is admitted by this review. Identical names, strings, plans, or other
  values do not admit cross-generation sharing; generation overlap must retain
  independent lifetime and replacement behavior unless later evidence reopens
  that ownership boundary explicitly.
- Resource snapshots retain bounded memory ownership and no ambient filesystem
  or network authority under ADR-0002.
- Representation-specific access remains inside its owner. The audit must not
  manufacture a generalized provider API or defeat AR-0007's future strategy
  seam.
- Diagnostics, source locations, work accounting, cancellation charge points,
  deterministic results, and host-mode parity are conservation requirements.
- Optimized prepared forms must preserve or improve deterministic attribution of
  retained and peak memory to the owning generation, prepared input, worker, or
  invocation. Faster execution does not justify memory that the host can no
  longer bound, observe, or retire predictably.
- The supported Rust facade must not expose arena indexes, tags, interner keys,
  plan opcodes, cache keys, or scratch-buffer ownership merely to enable an
  optimization.
- Safe Rust remains the reference. Self-validation at construction or admission
  can establish optimized-path preconditions, but it does not by itself admit
  `unsafe`; any unsafe representation still requires a separate accepted ADR
  satisfying ADR-0003.

## Candidate audit areas

The audit may examine, without committing to any of them:

- compact node identifiers and arena organization, including array-of-structs
  versus struct-of-arrays layouts;
- interned expanded names, prefixes, namespace URIs, and repeated string values;
- stylesheet-activated indexes for names, axes, keys, and template dispatch;
- pre-resolved name tests, static contexts, constants, and specialized plan
  operations selected during compilation rather than hot-loop feature checks;
- sequence/value representations for empty, singleton, small, homogeneous, and
  general sequences;
- document-order and subtree metadata that can make admitted navigation cheaper;
- reusable worker-local scratch buffers whose lifetime and clearing rules cannot
  leak invocation state;
- result construction and serialization buffers, including the future
  distinction between Unicode text and encoded bytes; and
- compact or compressed prepared forms where decode cost, random access, and
  retained memory are measured together.

This list is an inventory of questions, not an implementation plan.

## Rust-level opportunities

The audit is not limited to choosing a familiar container. Rust is low-level
enough that the same logical representation can have materially different
physical and execution behavior. The investigation may therefore examine:

- ownership and lifetime arrangements that remove cloning, reference counting,
  or synchronization from an admitted hot path while preserving immutable
  shared state;
- exact-capacity construction followed by slices or boxed slices where growth
  is no longer required;
- enum, option, newtype-index, and tagged-value layouts, including whether
  compiler niche optimization helps in the actual target builds;
- array organization, alignment, cache-line use, prefetch behavior, and false
  sharing between concurrent workers;
- safe arena, bump-allocation, small-inline, bit-set, and compact-string
  implementations, including the unsafe surface and maintenance quality of any
  dependency that provides them;
- static dispatch, monomorphized operations, generated specialized evaluators,
  function tables, or compact opcodes selected at compile/prepare time;
- bounds-check elimination that the optimizer can already prove from safe code,
  before considering unchecked indexing;
- worker-local reuse of capacities and buffers with explicit clearing,
  poisoning, and maximum-retention rules;
- atomics, locks, channels, and reference-count traffic that may be avoidable by
  assigning stronger ownership to a generation or worker; and
- compiler output, target features, and portable SIMD opportunities after a
  profile identifies a sufficiently important loop.

Layout intuition is not evidence. `size_of`, generated assembly, hardware
counters, and microbenchmarks can explain a mechanism, but admission still
requires end-to-end semantic, memory, latency, and throughput evidence on
supported targets. Compiler-specific layout accidents must not become persisted
formats, ABI promises, or soundness assumptions. `repr(packed)`, unchecked
indexing, raw pointers, custom allocation, intrinsics, or similar techniques do
not bypass ADR-0003 merely because the surrounding representation validates
itself.

## Initial profiling hypotheses

The following are high-probability places to measure, not findings about the
current implementation and not permission to optimize them before measurement:

1. **Prepared-XDM byte anatomy.** Attribute retained bytes to node records,
   text/string storage, expanded names and namespaces, parent/child relations,
   indexes, vector capacity slack, and ownership/reference-count overhead.
   Report source bytes, node count, retained bytes per node, peak construction
   bytes, and final prepared bytes rather than one unexplained expansion ratio.
2. **Warm execution allocation churn.** Count allocation operations and bytes
   during execution separately from resource import, XML/XDM construction,
   compilation, preparation, and result transfer. Tiny warm transforms make
   short-lived `Vec`, `String`, clone, and result-building costs plausible, but
   only phase-separated measurement can establish that they matter.
3. **XPath sequence shapes.** Record bounded histograms for sequence length and
   item kind, distinguishing empty, singleton, small, homogeneous node, and
   general sequences. A specialized representation is justified only if the
   observed distribution and affected operations support it.
4. **Name and namespace duplication.** Compare occurrence count, unique count,
   and retained bytes for local names, prefixes, namespace URIs, expanded names,
   and repeated string values. Interning must earn its lookup, ownership, and
   generation-lifetime costs.
5. **Reference-count and synchronization traffic.** Attribute `Arc` clones and
   drops, atomic operations, locks, and channels to compile, prepare, generation,
   worker, invocation, and result paths. Stronger ownership is preferable only
   when it preserves host lifecycle and independent generation retirement.
6. **Selection and navigation fan-out.** For template dispatch, observe
   candidates considered per selection. For path steps, observe nodes scanned
   versus nodes returned, classified by semantic operation. These ratios can
   justify stylesheet-activated indexes or pre-resolved tests without assuming
   that every document needs them.
7. **Scratch-capacity behavior.** Record requested size, reallocations,
   high-water mark, average use, retained capacity, and post-failure state per
   worker-local buffer. One unusually large invocation must not silently make
   its peak capacity permanent across the worker pool; trimming and maximum-
   retention policy require workload evidence.

The first recommended probes are phase-attributed Rust allocation/retention,
sequence length/item-kind histograms, and prepared-XDM byte anatomy. Together
they discriminate whether the earliest pressure is primarily temporary work,
value representation, retained document layout, or none of those. Allocation
and ownership improvements should be explored before unsafe code or SIMD unless
a profile points elsewhere.

Instrumentation must be explicitly supplied, bounded, and semantically inert.
It must not make ambient global telemetry part of execution semantics, leak
private values, alter case selection, or contaminate ordinary performance
measurements. Instrumented and uninstrumented paths need parity checks, and
measurement overhead must be reported when observations are used as evidence.

## Alternatives

### Retain straightforward reference representations

Keep the current safe structures unless profiles identify a material problem.
This minimizes preparation, complexity, and correctness risk and is the default
outcome if experiments do not demonstrate consumer-visible value.

### Add safe specialized representations behind private owners

Construct validated immutable forms during compile or prepare, then select a
lean execution path that does not repeatedly branch on unused features. Retain
the safe reference behavior for differential verification. Specialization may
be workload- or feature-shaped but must have explicit preparation and retention
budgets.

### Add optional indexes or caches

Build only indexes activated by compiled semantic knowledge or measured reuse.
This may trade preparation and memory for execution speed. Admission policy,
eviction, identity, concurrency, and cache ownership require evidence and may
reopen AR-0009. This review does not admit cross-generation sharing even when
content or interned values compare equal.

### Admit an unsafe optimized representation

Consider only after a safe specialized prototype leaves a measured requirement
unmet. This alternative is unavailable through this review alone and requires a
separate ADR-0003 exception with exact invariants, tools, benefit, and removal
criteria.

## Experiment method

1. Inventory the current logical data flow and ownership from resource bytes
   through XML, XDM, compilation, preparation, execution, and serialization.
2. Profile representative cold, prepared, warm, batch, and concurrent workloads
   before choosing a container. Attribute CPU, allocation, retained bytes,
   locality where observable, and result-transfer cost to their owning phase.
3. State one representation or Rust-level mechanism hypothesis and its expected
   tradeoff. Change one material variable at a time behind a private boundary.
4. Compare against the safe reference using semantic results, structured
   diagnostics, budgets, cancellation, and adversarial boundaries—not throughput
   alone.
5. Measure preparation latency, break-even reuse count, retained and peak memory,
   warm throughput, p50/p95/p99 latency, concurrency scaling, and host-visible
   behavior. Verify that memory remains attributable to the generation,
   prepared input, worker, or invocation that owns its lifetime.
6. Remove or retain the experiment according to evidence. A dead end is a valid
   result when its workload, measurements, and rejected hypothesis are recorded.

Microbenchmarks may locate pressure but cannot establish an engine or consumer
benefit by themselves.

## Findings and uncertainties

Compile-once and prepared-input reuse demonstrate that representation work can
matter. They do not identify the next representation or prove that a custom data
structure is beneficial. The strongest current direction is compile/prepare-
time specialization that activates only required machinery, because that fits
the existing lifecycle without adding repeated hot-path feature checks.

Unknowns include representative consumer distributions, reuse counts, source
shapes, namespace/name repetition, axis and template-selection pressure,
allocation ownership inside Rust, cache behavior, and the memory budget an
embedded ASP.NET consumer will tolerate per generation or worker.

## Disposition

**Incubating.** Preserve the audit and candidate inventory, but select no data
structure, index, cache, representation, unsafe exception, or public type. Normal
standards-driven implementation continues until profiles or consumer workloads
provide a concrete hypothesis to test.

## Required follow-up

- [ ] Capture at least one representative consumer workload with semantic
  fidelity sentinels, reuse count, concurrency, and memory/latency budgets.
- [ ] Add Rust-side allocation and retained-memory attribution for compile,
  prepare, execute, and serialize phases.
- [ ] Add bounded sequence length/item-kind histograms and prepared-XDM byte
  anatomy sufficient to distinguish node, name/namespace, text, relationship,
  index, capacity-slack, and ownership overhead.
- [ ] Add focused fan-out, duplication, reference-count/synchronization, and
  scratch-capacity probes only where the first profiles or representative
  workloads nominate them.
- [ ] Verify each experiment preserves deterministic retained/peak attribution
  and independent old/new generation retirement; do not use content equality to
  justify hidden cross-generation ownership.
- [ ] Produce a current representation and lifetime inventory without exposing
  it through the public facade.
- [ ] Include ownership, cloning/reference-count traffic, allocation shape,
  synchronization, layout, and generated-code observations in the inventory
  where profiles nominate them.
- [ ] Profile the reference path and nominate one measured representation
  hypothesis, or record that no material representation pressure was found.
- [ ] Prototype the nominated hypothesis in safe Rust and differentially verify
  it before considering any optimized or unsafe successor.
- [ ] Record negative experiments so later work does not repeat attractive dead
  ends without new evidence.

## Reopening triggers

- A representative workload attributes material time, allocation, or retained
  memory to a named representation or navigation operation.
- Prepared-state memory prevents the required worker count, generation overlap,
  or source distribution from meeting a consumer budget.
- The same compiled feature subset repeatedly pays for inactive machinery in a
  measured hot path.
- AR-0012 facade work needs an ownership or result type that private
  representation choices could accidentally constrain.
- A safe prototype demonstrates a valuable result or leaves a named requirement
  unmet in a way that may justify a separate unsafe exception review.

## Review history

- 2026-08-27 -- Opened as Incubating to preserve a future evidence-driven audit
  of prepared representations and data layout without selecting an optimization.
- 2026-08-27 -- Peer review made cross-generation sharing explicitly unadmitted
  and deterministic memory attribution a conservation requirement.
- 2026-08-27 -- Peer review added seven initial profiling hypotheses and
  prioritized phase-attributed allocation/retention, sequence-shape histograms,
  and prepared-XDM byte anatomy as the first discriminating probes.
