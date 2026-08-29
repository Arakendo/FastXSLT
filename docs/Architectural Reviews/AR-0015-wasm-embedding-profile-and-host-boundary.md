# AR-0015: WASM Embedding Profile and Host Boundary

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-28 |
| Last reviewed | 2026-08-28 |
| Scope | WebAssembly target, host resource boundary, retained lifecycle, controls, results, diagnostics, and parity |
| Trigger | WASM is reported as real future consumer pressure rather than generic portability optionality |
| Related ADRs | ADR-0002, ADR-0003, ADR-0005, ADR-0007 |
| Related reviews | AR-0002, AR-0009, AR-0010, AR-0012, AR-0014 |
| Related evidence | `../Evidence/private-prepared-input-reuse-2026-08-25.md`; `../Evidence/aspnet-host-mode-guarantee-cost-matrix-2026-08-26.md` |

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

## Alternatives

### Browser-oriented `wasm32-unknown-unknown`

This directly pressures JavaScript-visible byte transfer, synchronous versus
asynchronous API shape, browser memory limits, and package tooling. It has no
ambient filesystem and fits the presealed authority model well. It may require
binding/generated-code dependencies and does not represent server-side WASM
deployment or component interfaces.

### WASI-oriented module

A WASI host may suit server, plugin, or command-style consumers and can provide
stronger runtime containment. Its available I/O capabilities must still remain
host-owned rather than becoming engine fallback. Runtime-specific interruption
and resource controls would need explicit guarantee mapping.

### Component-model interface

A typed component interface could make ownership and structured diagnostics
clearer than a hand-built linear-memory ABI. Selecting it now would add tooling
and versioning commitments before a real consumer identifies its runtime and
distribution requirements.

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

## Disposition

Keep AR-0015 **Incubating**. Preserve WASM as real consumer pressure and a
bounded future experiment, but do not add it to the current critical path or
claim a supported target until the consumer identifies a runtime and supplies
representative lifecycle evidence.

No target, binding framework, public API, concurrency contract, resolver
profile, hard-containment guarantee, or performance expectation is selected.

## Required follow-up

- [ ] Obtain the consumer's runtime, target, deployment, trust, concurrency,
  stylesheet/resource, result, memory, and performance requirements.
- [ ] Inventory workspace dependencies and feature gates for the candidate
  target; record any native threads, atomics, filesystem, clocks, randomness,
  panic, or platform assumptions rather than hiding them behind conditional
  compilation.
- [ ] Compile the safe core and one no-I/O smoke transform for the selected
  target before designing a broad binding.
- [ ] Exercise a presealed multi-resource case, preferably the admitted
  `include-0401` slice, without filesystem or network fallback.
- [ ] Prove compile-once/prepared reuse across calls within one instance and
  deterministic release/replacement of the owning generation.
- [ ] Differentially compare result bytes or text and every structured
  diagnostic field with direct Rust for the same positive, unsupported,
  invalid, denied, cancelled, and budget-exhausted cases.
- [ ] Measure module load, resource copy, compilation, preparation, warm
  execution, result transfer, retained/peak linear memory, and reuse break-even
  separately from native execution.
- [ ] Decide through a later ADR whether any evidenced target and boundary
  becomes supported; keep target-specific operational guarantees explicit.

## Reopening triggers

- A consumer supplies a named WASM runtime and representative transform.
- A dependency or Rust target limitation prevents the safe core from building.
- Linear-memory retention or 32-bit accounting changes resource limits or the
  prepared-input lifecycle.
- The host requires live resources, async callbacks, shared-memory threads,
  reentrancy, component packaging, or hard interruption guarantees.
- WASM performance or copy cost pressures a different result/resource boundary.

## Review history

- 2026-08-28 -- Opened as Incubating from stated future consumer pressure. The
  first candidate is a presealed, memory-resident parity experiment; no WASM
  target or supported profile was selected.
