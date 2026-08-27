# CR-0001: Tokimu Web3D X3D-to-VRML Transformation

| Field | Value |
| --- | --- |
| Status | Proposed |
| Requested by | Tokimu |
| Opened | 2026-08-26 |
| Last reviewed | 2026-08-26 |
| Target | Rust embedding facade and consumer-driven standards implementation queue |
| Consumer owner | Tokimu |
| Related reviews | AR-0001, AR-0004, AR-0005, AR-0009, AR-0010, AR-0012 |
| Related ADRs | ADR-0002, ADR-0005, ADR-0007 |
| Related plans | `docs/Plans/roadmap.md` |

## Consumer problem

Tokimu is a separate Rust project that consumes X3D content. A future import
path may need Web3D's published `X3dToVrml97.xslt` transformation to produce
VRML97 output before Tokimu validates and imports it.

The current independent Tokimu investigation uses Saxon and has produced a
non-authoritative result that omitted selected authored field values. Tokimu
has quarantined that output. The documented Web3D invocation appears to involve
processor options, resource/catalog behavior, and stylesheet parameters beyond
merely naming a stylesheet. FastXSLT must not be used to conceal or reinterpret
that unresolved fidelity problem.

This request records a representative consumer workload. It does not change
FastXSLT's selected profile, make the complete Web3D stylesheet an immediate
acceptance gate, or promise compatibility with Saxon-specific behavior.

## Consumer pipeline

```text
Tokimu-owned X3D, stylesheet, supporting resources, parameters, and policy
    -> narrow Tokimu/FastXSLT adapter
    -> bounded resource admission and sealed snapshot
    -> reusable compiled Web3D stylesheet
    -> prepared X3D input where justified
    -> invocation-local transformation
    -> in-memory VRML result and structured diagnostics
    -> Tokimu validation, import, and optional publication
```

## Ownership boundary

### Consumer owns

- The authoritative Web3D invocation, resource acquisition, catalog inputs,
  trust policy, and any host filesystem or network access.
- X3D/VRML domain meaning, semantic sentinels, output validation, and the
  decision to persist or import a result.
- Selection of parameters, cancellation, budgets, reuse policy, and deployment
  or process-isolation policy.

### FastXSLT owns

- Implemented XSLT, XPath, XDM, serialization, URI, and diagnostic semantics.
- Bounded logical resource lookup inside explicitly supplied authority.
- Immutable compiled state, invocation isolation, structured outcomes, and
  documented work-limit behavior.

### FastXSLT must not need to understand

- Tokimu's scene, WAD, renderer, asset, or domain-model types.
- Web3D-specific meaning that is not defined by an admitted standard.
- Saxon command-line conventions as though they were XSLT semantics.

## Requested contract

The required eventual behavior is a supported Rust-native facade that can:

- admit stylesheet, source, catalog-related, and supporting resource bytes with
  logical identity and relevant base-URI/provenance information;
- seal bounded authority without retaining or reopening host file handles;
- compile the Web3D stylesheet once and reuse immutable compiled state;
- accept invocation-local stylesheet parameters, cancellation, and budgets;
- transform one or more X3D inputs without coupling Tokimu to parser events,
  XDM arena identifiers, optimizer IR, caches, or private runtime types;
- return an in-memory semantic or serialized result without inferring a host
  destination; and
- distinguish invalid XML/stylesheet, unsupported capability, standards error,
  missing or denied resource, parameter failure, limit, cancellation, and
  internal failure without display-string parsing.

Rust-native in-process embedding is preferred. Process isolation remains an
optional host/security profile rather than an inherent requirement of Tokimu.

## Existing evidence

- Tokimu is a concrete Rust consumer with an X3D-to-VRML workflow.
- Web3D publishes `X3dToVrml97.xslt` and documents a Saxon-oriented conversion
  path, but its authoritative invocation has not yet been reproduced here.
- A local conversion dropped authored values and is explicitly quarantined.
- FastXSLT already has private memory-resident resource, compile/reuse,
  prepared-input, structured-diagnostic, budget, cancellation, and in-memory
  result experiments. It does not expose a supported Rust facade.
- The exact stylesheet revision, license/redistribution terms, complete resource
  graph, parameter set, representative inputs, trusted outputs, semantic
  sentinels, and workload distribution remain missing evidence.

## Acceptance evidence

| Case | Pressure | Expected result | Evidence |
| --- | --- | --- | --- |
| Representative success | Pinned Web3D stylesheet and X3D input | In-memory VRML result with trusted sentinels preserved | Pending |
| Rust lifecycle | Tokimu-shaped adapter | Admit, seal, compile once, transform many, and release through supported types | Pending; AR-0012 |
| Invalid input | Malformed X3D XML | Structured invalid-input outcome with location | Pending |
| Unsupported behavior | Valid unimplemented stylesheet construct | Distinct unsupported capability and stable source identity | Pending |
| Missing resource | Referenced but unadmitted dependency | Missing-resource outcome without ambient fallback | Pending |
| Denied resource | Resolver policy denies dependency | Denied-resource outcome without filesystem/network access | Pending |
| Parameter problem | Missing, invalid, or unknown required parameter | Structured parameter/static/dynamic outcome as applicable | Pending |
| Limit or cancellation | Bounded realistic Web3D work | Classified termination without poisoning reusable compiled state | Pending |
| Handle release | File-backed Tokimu import adapter | Original files replaceable after admission | Existing generic evidence; workload-specific evidence pending |
| Semantic fidelity | Authored field sentinels | Match independently trusted Web3D/reference evidence | Pending |
| Reuse and performance | Repeated representative conversions | Separate cold compile, preparation, warm execution, result, allocation, and retention measurements | Pending after correctness |

## Compatibility and migration

There is no supported Rust API to migrate today. This request pressures the
smallest facade over the existing lifecycle; it does not stabilize private
types, compiled artifact formats, corpus storage, ABI layouts, diagnostic text,
or Tokimu adapter types. Later facade decisions require AR-0012 and, where they
create a durable public contract, an accepted ADR.

## Security and resource limits

X3D, stylesheets, catalog mappings, and supporting resources may be untrusted.
Tokimu grants explicit bytes and logical resolution authority. FastXSLT must not
inherit ambient filesystem/network/entity authority or expose sensitive host
paths by default. XML/XDM/XPath/XSLT/result/serialization budgets and
cooperative cancellation remain explicit; hard termination requires an
isolated execution profile and must not be implied by an in-process timeout.

## Explicit non-goals

- Tokimu-specific transformation instructions or X3D/VRML semantics.
- Treating Saxon behavior as the definition of FastXSLT behavior.
- Admitting the quarantined incomplete output as expected corpus data.
- Immediate broad support for every construct in `X3dToVrml97.xslt`.
- Engine-owned file discovery, network access, catalog mutation, output paths,
  or result publication.
- A performance comparison before authoritative correctness is reproduced.

## Proposed disposition

Retain as **Proposed** consumer and implementation-queue evidence. Investigate
the supported Rust lifecycle through AR-0012. Before planning Web3D execution,
Tokimu must reproduce the authoritative pipeline and FastXSLT must record the
pinned stylesheet revision and license, explicit resource graph and parameters,
representative input, trusted result or sentinels, and the first unsupported
standards frontier.

Required capabilities should then enter ordinary standards-driven vertical
slices. Periodic compilation may move the unsupported frontier, but only
sentinel and complete-result evidence may establish consumer fidelity.

## Completion condition

Mark this request Implemented only when Tokimu can use a supported Rust-native,
memory-resident, explicitly bounded facade to execute the pinned Web3D workflow;
resources and invocation parameters are explicit; structured failures require
no string parsing; compiled state is safely reusable; and independently trusted
semantic sentinels and complete expected output pass repeatably.

## History

- 2026-08-26 -- Opened as Proposed from Tokimu/Web3D consumer pressure; the
  incomplete local output remains quarantined and no compatibility claim is
  made.
