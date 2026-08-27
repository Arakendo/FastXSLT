# CR-0001: Tokimu Web3D X3D-to-VRML Transformation

| Field | Value |
| --- | --- |
| Status | Deferred |
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

The independent Tokimu investigation isolated a stylesheet-version fidelity
problem rather than an XSLT-processor defect. Web3D stylesheet revision `40046`
deterministically omitted selected authored fields under the existing
Saxon-HE 10.9 pipeline. Tokimu reports that the published working stylesheet
matches immutable SVN revision `35289`, which preserves its selected values.
The older output remains unsuitable as expected corpus data. FastXSLT must not
conceal or reinterpret a stylesheet fidelity problem merely because execution
is reproducible.

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
  path. Tokimu reports revision `35289` as its known-good immutable reference
  and revision `40046` as deterministically omitting authored fields. FastXSLT
  has not independently reproduced the complete invocation here.
- Tokimu regenerated 13 derived VRML97 fixtures from revision `35289` and added
  executable fidelity sentinels for translations, indexed topology and
  coordinates, texture URLs, material colours, and interpolator keys/values.
  Those consumer-owned checks are future acceptance evidence, not FastXSLT
  conformance evidence.
- FastXSLT already has private memory-resident resource, compile/reuse,
  prepared-input, structured-diagnostic, budget, cancellation, and in-memory
  result experiments. It does not expose a supported Rust facade.
- The stylesheet candidate and consumer sentinel categories are now known.
  License/redistribution terms, complete resource graph, parameter set, fixture
  provenance available to FastXSLT, complete trusted outputs, and workload
  distribution remain missing evidence in this repository.

## Acceptance evidence

| Case | Pressure | Expected result | Evidence |
| --- | --- | --- | --- |
| Representative success | Web3D revision `35289` and representative X3D input | In-memory VRML result with Tokimu's trusted sentinels preserved | Consumer evidence exists; FastXSLT reproduction pending |
| Rust lifecycle | Tokimu-shaped adapter | Admit, seal, compile once, transform many, and release through supported types | Pending; AR-0012 |
| Invalid input | Malformed X3D XML | Structured invalid-input outcome with location | Pending |
| Unsupported behavior | Valid unimplemented stylesheet construct | Distinct unsupported capability and stable source identity | Pending |
| Missing resource | Referenced but unadmitted dependency | Missing-resource outcome without ambient fallback | Pending |
| Denied resource | Resolver policy denies dependency | Denied-resource outcome without filesystem/network access | Pending |
| Parameter problem | Missing, invalid, or unknown required parameter | Structured parameter/static/dynamic outcome as applicable | Pending |
| Limit or cancellation | Bounded realistic Web3D work | Classified termination without poisoning reusable compiled state | Pending |
| Handle release | File-backed Tokimu import adapter | Original files replaceable after admission | Existing generic evidence; workload-specific evidence pending |
| Semantic fidelity | Translation, indexed topology/coordinates, texture URL, material colour, and interpolator sentinels | Match Tokimu's independently trusted revision-`35289` evidence | Consumer sentinels exist; FastXSLT execution pending |
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

## Disposition

Defer consumer-specific execution work while Tokimu uses Saxon as its likely
near-term processor. Retain this request as future consumer and
implementation-queue evidence; Saxon remains a tooling/reference path rather
than the definition of FastXSLT semantics.

Reopen planning when Tokimu has a concrete reason to replace or supplement
Saxon. Revision `35289` and Tokimu's five sentinel categories are the current
reference candidate. FastXSLT must then independently record its license,
explicit resource graph and parameters, representative input, trusted result,
and the first unsupported standards frontier. AR-0012 continues to own the
general Rust facade question independently of this request's schedule.

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
- 2026-08-26 -- Deferred because Tokimu is likely to use Saxon in the near term;
  retain the workload for later standards, facade, fidelity, and performance
  evidence.
- 2026-08-26 -- Recorded Tokimu's finding that revision `40046` reproducibly
  loses authored fields while immutable revision `35289` preserves its selected
  values under Saxon-HE 10.9; five consumer-owned fidelity sentinel categories
  now define the future correctness target without reopening the request.
