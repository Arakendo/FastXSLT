# FastXSLT Roadmap

The roadmap is ordered by executable semantic evidence, not by a desire to fill
every conceptual module. Dates are intentionally absent until requirements and
standards scope are decided.

## M0 -- Project scaffold

- [x] Buildable Rust workspace with formatting, lint, test, and docs gates.
- [x] Documentation authority and lifecycle established.
- [x] Initial SDD, ADR process, AR process, and testing strategy established.
- [x] Golden corpus layout seeded without claiming executable support.

Exit criterion: a clean checkout can run the local verification script and the
next architectural questions are visible rather than encoded accidentally.

## M1 -- Standards decision and first vertical slice

- [ ] Close AR-0001 through an accepted standards-profile ADR.
- [ ] Select and document the XML parser boundary for the slice.
- [ ] Compile one root template, evaluate one path/value expression, and produce
  one result through a private end-to-end engine path.
- [ ] Run `corpus/golden/hello` with structured diagnostics.
- [ ] Load the golden source and stylesheet through a bounded resource set,
  seal it, and execute the case without engine-owned filesystem access.
- [ ] Release import handles before sealing, then replace or remove the original
  fixture files and prove the snapshot still executes identically.
- [ ] Add negative cases that distinguish invalid input from unsupported syntax.
- [ ] Establish the first structured boundary failures and reportable semantic
  outcomes from emitted behavior rather than an aspirational error catalog,
  providing evidence for AR-0004.

Exit criterion: the seed transform passes through the intended layers and every
implemented behavior belongs to a named standards slice.

## M2 -- Data model and XPath foundation

- [ ] Define node identity, document order, names, strings, and sequence/value
  behavior needed by the accepted profile.
- [ ] Record the navigation and retention capabilities the implemented XPath
  and XSLT slice actually requires; keep tree-specific random-access assumptions
  inside their physical owner, providing evidence for AR-0007.
- [ ] Expand XPath lex/parse/evaluate tests before growing XSLT instructions.
- [ ] Establish diagnostic codes and source spans across XML and XPath phases.
- [ ] Provide a read-only semantic inspection snapshot for the implemented
  compilation slice without exposing private parser, arena, or IR types,
  providing evidence for AR-0005.
- [ ] Import the first licensed, versioned, integrity-checked suite selection.

Exit criterion: a published test report identifies supported, unsupported,
failed, and harness-error cases without an unqualified conformance claim.

## M3 -- Reusable stylesheet engine

- [ ] Separate reusable compiled stylesheet state from dynamic transform state.
- [ ] Add template selection, built-in rules, parameters, variables, and output
  behavior required by the accepted profile.
- [ ] Establish explicit URI/resource resolution and execution limits.
- [ ] Execute a batch of independent requests with shared compiled stylesheets
  and isolated dynamic contexts; randomize scheduling, correlate results by
  identity, and prove a batch of one matches the convenience API.
- [ ] Compare parse-per-invocation with private snapshot/work-generation prepared
  input reuse, reporting retained XDM and peak construction memory separately.
- [ ] Execute a two-stage host-owned workflow and prove stage-one results remain
  invisible until explicitly admitted into a stage-two snapshot.
- [ ] Compare file-per-call, preloaded snapshot, warmed filesystem cache, and
  compile-once paths with correctness held constant.
- [ ] Add differential and integration tests against named processors.
- [ ] Run an ASP.NET consumer workbench through the selected host boundary,
  reusing compiled stylesheets across requests with explicit cancellation and
  resource policy.

Exit criterion: representative stylesheets compile once, transform multiple
documents without leaked state, fail through structured diagnostics, and expose
measured end-to-end behavior to at least one non-Rust consumer.

## Later candidates

CLI, WASM, streaming implementation or conformance, schema awareness, extension
functions, packages, alternate execution backends, transformation graphs, and
specific parallel executor strategies require their own product evidence and
architectural review. Their presence in this list is not a commitment.
