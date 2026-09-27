# AR-0015: WASM Embedding Profile and Host Boundary

| Field | Value |
| --- | --- |
| Status | Accepted through ADR-0020 |
| Opened | 2026-08-28 |
| Last reviewed | 2026-09-24 |
| Scope | WebAssembly target, host resource boundary, retained lifecycle, controls, results, diagnostics, and parity |
| Trigger | WASM is reported as real future consumer pressure rather than generic portability optionality |
| Related ADRs | ADR-0002, ADR-0003, ADR-0005, ADR-0011, ADR-0016 |
| Related reviews | AR-0002, AR-0009, AR-0010, AR-0012, AR-0013, AR-0014, AR-0018 |
| Related evidence | `../Evidence/private-prepared-input-reuse-2026-08-25.md`; `../Evidence/aspnet-host-mode-guarantee-cost-matrix-2026-08-26.md`; `../Evidence/wasm-browser-feasibility-and-benchmark-baseline-2026-09-24.md` |

## Architectural question

Can FastXSLT support a presealed, memory-resident WASM embedding profile using
the same compilation and transformation semantics as direct Rust and the .NET
workbenches, without introducing a second engine or granting ambient host
authority?

## Trigger and evidence

WASM is now a stated future consumer need, but the consumer has not yet supplied
its exact runtime, deployment target, stylesheet/resource graph, workload,
memory ceiling, trust model, or latency budget. That is enough to preserve and
investigate a boundary, but not enough to select browser JavaScript,
`wasm32-wasip1`, a component-model host, or another runtime as the supported
profile.

Existing evidence is encouraging but indirect. FastXSLT already accepts owned
bytes, seals qualified resources in memory, separates compilation from
invocation, reuses compiled and prepared state, returns bounded serialized
results, preserves structured diagnostics, and keeps host adapters outside the
semantic engine. None of that proves that the workspace dependencies compile
for a WASM target, that retained XDM fits linear-memory ceilings, or that a
particular binding mechanism has acceptable copy and call costs.

A later design discussion sharpened the likely consumer value: a browser-hosted
XML/XSLT workspace could execute local transformations without uploading source
documents to a service. That is credible pressure for a browser-oriented first
probe, not evidence that `wasm32-unknown-unknown` should become the first
supported target. Server/plugin WASI and component-model consumers have
different capability, packaging, interruption, and lifecycle requirements and
must not be silently treated as the same product.

## Ownership and constraints

- FastXSLT owns XML/XDM/XPath/XSLT semantics, compilation, prepared state,
  deterministic work limits, structured diagnostics, and serialization.
- The embedding host owns module instantiation, byte acquisition, ambient
  authority, deployment, instance lifetime, scheduling, interruption, and
  publication of results.
- ADR-0002 requires resources to be copied or otherwise explicitly admitted
  before compilation and keeps core execution memory-resident. WASM imports do
  not create permission to fetch a URL-shaped logical identity.
- ADR-0005 keeps batch members independent and unordered. A WASM convenience
  call remains a batch of one rather than a second execution model.
- AR-0014 keeps reference resolution separate from acquisition authority. A
  first WASM profile uses a presealed resource closure and no live resolver.
- AR-0010's guarantee classes remain distinct. Work budgets and cancellation
  are cooperative engine controls; host-specific epoch interruption, fuel, or
  instance destruction cannot silently become a portable FastXSLT guarantee.
- The existing safe semantic path remains the parity reference. A WASM adapter
  does not authorize unsafe code or a target-specific semantic backend.
- ADR-0016's host-owned-policy principle still applies: the host supplies
  externally meaningful resource and concurrency ceilings; FastXSLT owns the
  internal implementation used to enforce them. A WASM binding must not invent
  hidden defaults and advertise them as hard runtime or browser limits.
- AR-0013 requires prepared-representation changes to be measured against the
  safe reference. Linear-memory pressure may nominate representation work, but
  does not itself admit a packed arena, global interning, cross-instance
  sharing, or unsafe specialization.

## Candidate first slice

The first viability experiment should remain deliberately narrow:

```text
host-owned identities and bytes
              |
              v
bounded sealed resource snapshot
              |
              v
compile stylesheet and prepare source
              |
              v
one transform or bounded independent batch
              |
              v
bounded result plus structured diagnostic fields
```

The experiment may retain compiled/prepared state inside one WASM instance
across calls. It makes no same-instance concurrency promise and admits no live
filesystem/network resolver, async callback, borrowed host buffer, process-like
hard-kill guarantee, persisted compiled artifact, or cross-instance cache.

A binding-neutral lifecycle sketch is:

```text
create instance
    -> admit owned (logical identity, bytes) resources
    -> seal snapshot
    -> compile stylesheet by logical identity
    -> prepare source by logical identity
    -> transform once or transform a bounded independent set
    -> return owned result bytes/text or structured diagnostic fields
    -> release prepared, compiled, snapshot, and instance state explicitly
```

The first adapter may copy host buffers into linear memory and copy serialized
results back out. Those copies must be visible in measurements rather than
hidden behind a zero-copy claim. It must not retain a JavaScript, WASI, or other
host-language borrow across calls. A batch-of-one and sequential transform-many
path must use the same semantic engine and ownership rules.

## Initial non-goals

- no implicit `fetch`, filesystem, socket, environment, clock, or random
  authority;
- no live or asynchronous resource resolver;
- no public XDM, AST, execution-plan, or linear-memory layout;
- no shared-memory or same-instance multithreaded execution contract;
- no persistent or cross-instance compiled/prepared cache;
- no target-specific rewrite of XML, XPath, XSLT, or serialization semantics;
- no claim that WASM alone makes hostile stylesheets safe;
- no hard process-memory, browser-tab-survival, or exactly-once guarantee; and
- no stable wire, JavaScript, component, or language-binding ABI from the
  feasibility experiment.

## Alternatives

### Browser-oriented `wasm32-unknown-unknown`

This directly pressures JavaScript-visible byte transfer, synchronous versus
asynchronous API shape, browser memory limits, and package tooling. It has no
ambient filesystem and fits the presealed authority model well. It may require
binding/generated-code dependencies and does not represent server-side WASM
deployment or component interfaces.

This is the leading first feasibility candidate because it puts the strictest
pressure on ambient-authority assumptions and directly exercises the local,
no-upload workspace use case. The browser host would own every byte admitted to
the snapshot. An `xsl:include href="other.xsl"` reference would resolve only to
an already admitted logical resource; its URL-like spelling would never imply
Fetch API authority.

### WASI-oriented module

A WASI host may suit server, plugin, or command-style consumers and can provide
stronger runtime containment. Its available I/O capabilities must still remain
host-owned rather than becoming engine fallback. Runtime-specific interruption
and resource controls would need explicit guarantee mapping.

Preopened directories, sockets, clocks, and other WASI capabilities must not be
used directly by the semantic engine. A future WASI adapter may translate
explicit host authority into admitted resources, but the mere availability of
a WASI capability cannot become fallback acquisition behavior.

### Component-model interface

A typed component interface could make ownership and structured diagnostics
clearer than a hand-built linear-memory ABI. Selecting it now would add tooling
and versioning commitments before a real consumer identifies its runtime and
distribution requirements.

It remains attractive for a later typed multi-language boundary, especially
for owned byte lists, logical identities, bounded diagnostics, and handle
lifecycles. The feasibility probe should not spend its evidence budget designing
that public interface before the engine has rendered one unchanged stylesheet
through WASM.

### Reuse a Rust-to-WASM consumer directly

A Rust consumer could initially instantiate FastXSLT without a JavaScript or
component facade. This minimizes boundary invention but does not answer
cross-language transfer, packaging, or non-Rust host requirements.

### Defer all target work

The current host-neutral architecture can remain unchanged until the consumer
supplies a concrete target. This avoids speculative tooling but risks finding a
dependency, 32-bit accounting, or linear-memory problem only after a public
lifecycle begins stabilizing.

## Findings and uncertainties

The architecture already has the right semantic seam: host-supplied bytes feed
the same sealed snapshot, compiled program, prepared input, invocation, result,
and diagnostic lifecycle used elsewhere. A viability build should therefore
test an adapter and target constraints, not fork the engine.

The following remain unknown:

- exact target triple, runtime, component/binding toolchain, and packaging;
- dependency and feature compatibility, including synchronization assumptions;
- whether retained compiled/prepared state survives calls in the consumer's
  instance lifecycle;
- 32-bit length/conversion behavior and practical linear-memory ceilings;
- preparation inflation, peak memory, copy count, and reuse break-even point;
- synchronous, cooperative-cancellation, and host interruption behavior;
- result bytes versus strings and the structured diagnostic transport shape;
- single-instance reentrancy and whether multiple instances are the only
  bounded concurrency mechanism; and
- native-versus-WASM cold load, warm throughput, tail latency, and result-copy
  cost for a semantically identical workload.

Prepared XDM is the leading memory risk. Existing native measurements already
show that prepared representations can retain materially more memory than the
source bytes. A WASM experiment must therefore distinguish at least:

```text
raw admitted bytes
host-to-WASM copies
parse/XDM construction peak
retained prepared XDM
compiled stylesheet retention
invocation scratch/result construction
serialized result retention
WASM-to-host copies
```

The accounting must preserve ownership attribution to snapshot, compiled
generation, prepared input, invocation, and result where practical. Linear
memory size and runtime/process memory are observations, not substitutes for
the deterministic charges FastXSLT owns.

Boundary copying is the leading performance uncertainty. The first comparison
must report module initialization, host-to-WASM resource transfer, parsing,
compilation, preparation, warm execution, serialization, and result transfer
separately. An aggregate "WASM transform time" cannot justify optimizing the
semantic engine when the dominant cost may be moving bytes across the host
boundary.

The initial concurrency model should be one active invocation per instance.
Browser applications may experimentally use multiple Web Workers and one WASM
instance per worker; other hosts may instantiate a bounded pool. This is a host
topology experiment, not a promise that instances share compiled/prepared state
or that the engine internally uses WASM threads.

WASM can add containment in combination with explicit FastXSLT authority and
work limits, but the guarantee belongs to the selected runtime and capability
configuration. Linear-memory isolation does not prove termination, browser-tab
survival, confidentiality from a capability-bearing host, or hard reclamation
without instance disposal.

## Disposition

Close AR-0015 as **Accepted through ADR-0020**. The implemented safe-core build,
private adapter, sealed dependency, retained lifecycle, structured-control, and
unchanged standards-workload evidence are sufficient to select
`wasm32-unknown-unknown` as an official continuously built target.

The decision selects a build target and presealed architectural boundary only.
It does not stabilize the current binding framework or exports, select browser
or server packaging, promise a public API, widen the single-instance sequential
concurrency model, admit live acquisition, provide a hard-containment guarantee,
or make a competitive performance claim. Those require new consumer evidence
and a later review rather than keeping this feasibility question open.

## Required follow-up

- [ ] Obtain the consumer's runtime, target, deployment, trust, concurrency,
  stylesheet/resource, result, memory, and performance requirements.
- [ ] Inventory workspace dependencies and feature gates for the candidate
  target; record any native threads, atomics, filesystem, clocks, randomness,
  panic, or platform assumptions rather than hiding them behind conditional
  compilation.
- [x] Compile the safe core and one no-I/O smoke transform for the selected
  target before designing a broad binding.
- [ ] For the first browser-oriented probe, expose only owned byte admission,
  sealing, compile, prepare, sequential transform, result transfer, structured
  diagnostics, and explicit release. Keep generated glue and concrete exports
  private and replaceable.
- [x] Exercise a presealed multi-resource case without filesystem or network
  fallback. The first probe uses a synthetic relative `xsl:include` sentinel;
  the admitted `include-0401` slice remains part of the broader parity matrix.
- [ ] Prove compile-once/prepared reuse across calls within one instance and
  deterministic release/replacement of the owning generation.
- [ ] Differentially compare result bytes or text and every structured
  diagnostic field with direct Rust for the same positive, unsupported,
  invalid, denied, cancelled, and budget-exhausted cases.
- [ ] Include at least simple XSLT 1.0, sealed include/import, an XPath-heavy
  transform, namespace-heavy XML output, text serialization, HTML
  serialization, compile failure, runtime failure, budget exhaustion, and—only
  if the host supplies a sensible cooperative signal—cancellation in the parity
  matrix.
- [ ] Measure module load, resource copy, compilation, preparation, warm
  execution, result transfer, retained/peak linear memory, and reuse break-even
  separately from native execution.
- [ ] Measure binary/package size and 1 MiB, 10 MiB, and 100 MiB source shapes
  where the selected runtime can admit them safely; record graceful admission
  failure rather than forcing a size that exceeds the runtime envelope.
- [ ] Verify 32-bit conversions and every host-visible length before allocation
  or copy. Record whether dependencies introduce first-party or transitive
  unsafe code; do not claim a zero-unsafe artifact without tool evidence.
- [ ] Compare one-instance sequential reuse with a bounded multiple-instance
  host pool without selecting shared-memory threads or cross-instance state.
- [ ] Record whether a browser-local no-upload workflow is materially useful
  despite any native performance gap; native parity is a correctness oracle,
  not a requirement that WASM equal native throughput.
- [x] Decide through ADR-0020 that `wasm32-unknown-unknown` and the presealed
  shared-engine boundary are officially built; keep binding, packaging, and
  target-specific operational guarantees explicit and deferred.

## Reopening triggers

- A consumer supplies a named WASM runtime and representative transform.
- A dependency or Rust target limitation prevents the safe core from building.
- Linear-memory retention or 32-bit accounting changes resource limits or the
  prepared-input lifecycle.
- The host requires live resources, async callbacks, shared-memory threads,
  reentrancy, component packaging, or hard interruption guarantees.
- WASM performance or copy cost pressures a different result/resource boundary.
- A browser workspace, server/plugin host, or multi-language consumer supplies
  an exact distribution and lifecycle requirement that conflicts with the
  deliberately private feasibility adapter.

## Review history

- 2026-08-28 -- Opened as Incubating from stated future consumer pressure. The
  first candidate is a presealed, memory-resident parity experiment; no WASM
  target or supported profile was selected.
- 2026-09-24 -- Expanded the investigation from generic portability into a
  concrete browser-oriented feasibility candidate while preserving WASI and
  component-model targets as distinct alternatives. Added the minimal owned
  lifecycle, explicit non-goals, parity workload matrix, boundary-copy phase
  measurements, prepared-XDM/linear-memory accounting, single-instance lane,
  and runtime-specific containment caveats. Status remains Incubating; no
  supported target, public binding, or WASM guarantee is selected.
- 2026-09-24 -- Built the safe engine and a private `wasm-bindgen` adapter for
  `wasm32-unknown-unknown`, then executed it through Node.js. The probe retained
  compiled/prepared state, produced exact unchanged `for-004` results, resolved
  a relative include only from admitted memory, preserved representative budget
  and invalid-input diagnostics, and established an initial module/copy/warm-
  execution baseline. The result establishes feasibility only; browser support,
  complete parity, explicit release, larger memory envelopes, multi-instance
  scaling, and a public binding remain open.
- 2026-09-24 -- Accepted the evidenced target through ADR-0020. The repository
  now treats `wasm32-unknown-unknown` compilation as an official CI-enforced
  build constraint while leaving the current generated binding and all broader
  runtime/product contracts private.
