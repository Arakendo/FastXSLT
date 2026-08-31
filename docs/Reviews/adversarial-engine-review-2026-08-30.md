# FastXSLT Adversarial Engineering Review

Date: 2026-08-30  
Remediation updated: 2026-08-31
Scope: current repository state on the reviewed worktree  
Method: read-only source, decision-record, evidence, corpus, Rust, native ABI,
worker, and managed-host inspection; full workspace test execution; targeted
static counterexamples; TOML parser validation. No engine code was changed.

## Remediation status

This table tracks repository work performed after the read-only review. The
original findings remain below as reviewed; a completed status does not rewrite
their original evidence. Implementation details and validation are recorded in
[the first correctness-tranche evidence](../Evidence/adversarial-review-first-correctness-tranche-2026-08-30.md).

| Finding | Status | Current disposition |
| ---: | --- | --- |
| 1 | **Completed** | Both XSLT30 overlays are parsed as strict typed records. Duplicate/missing fields, duplicate case identities, incoherent dispositions, and cross-record pass assertions fail. The malformed rationale records were corrected. |
| 2 | **Completed** | Every admitted location-path step now normalizes repeated `NodeId` values into document order. Direct `/r/a/..` and template-dispatch regressions pass. |
| 3 | **Completed** | Temporary-tree `xsl:copy` now uses the compiled shallow-copy instruction path, including constructed attributes and body execution. The unconditional deep-copy shortcut was removed. |
| 4 | **Boundary closed** | FastXSLT now reports `FXRT1014 / unsupported` before strip-all execution when the source contains any `xml:space` declaration. Full inherited `preserve`/`default` semantics remain deliberately unimplemented under ADR-0012. |
| 5 | **Completed** | Source element copies retain effective in-scope namespace bindings assembled from their ancestor lineage. Isolated descendant `xsl:copy-of` and `xsl:copy` regressions pass. |
| 6 | **Partially repaired -- policy decision required** | Outcomes are bounded and creation publication is atomic. Separate probes now compare 100,000-handle abandonment with a 144-handle host-shaped high-water containing two ×4 prepared generations. [AR-0017](../Architectural%20Reviews/AR-0017-native-handle-registry-retention-and-abandonment.md) still requires consumer requirements and policy comparison before selecting count, byte, domain, shrink, or isolation behavior. |
| 7 | **Confirmed -- remediation decision required** | Test-only instrumentation proves exact `selected nodes × matched templates` fanout. A signal after candidate 1 remained unobserved through the other 128 simple-pattern candidates. The optimized sweep reached 33,024 candidates and a 429.3 us local median; no budget unit, check frequency, or index is selected yet. |
| 8 | **Completed** | Temporary path and built-in selections carry real focus position/size through template and `next-match` execution. Two-node `1/2`, `2/2` regression evidence passes. |
| 9 | **Boundary closed** | Same-module forward and cyclic global defaults fail at compilation as `FXST1044 / unsupported`; admitted backward dependencies continue to compile. A general dependency graph remains deferred. |
| 10 | **Completed** | Cancellation commands are assembled and serialized as complete bounded frames. A 10,000-pair byte-fragmenting stress probe recovered all 20,000 unique frames exactly once; the existing live-worker probe retains cancellation correlation and same-process recovery. |
| 11 | **Confirmed -- representation comparison required** | Widths 8/32/128/256 performed 8/32/128/256 full document-rooted path evaluations and exactly 81/1,089/16,641/66,049 charged node visits. One-less visit limits fail correctly, so accounting is honest; AR-0013 must compare a safe invocation-owned membership view before any optimization is admitted. |
| 12 | **Partially confirmed -- representation comparison required** | An eight-call chain with 256 globals clones 2,048 global entries and adds 8,824 allocation requests, 432,576 requested bytes, about 419 KiB peak live requested memory, and 598.3 us median over the same-global depth-zero control. AR-0013 may compare a safe overlay frame; prepared-XDM field duplication remains unmeasured. |

Current total: **8 findings handled**, comprising six semantic, evidence, or
operational repairs and two explicit unsupported boundaries. **Four findings
remain open**: two resource/accounting decisions, one confirmed representation
comparison, and one partially confirmed compound representation finding. The
first completed tranche is commit `95aa31a`; the
subsequent [worker control-frame evidence](../Evidence/aspnet-worker-control-frame-serialization-2026-08-31.md)
records Finding 10's stress and operational validation. The
[template-candidate evidence](../Evidence/template-candidate-fanout-and-cancellation-gap-2026-08-31.md)
advances Finding 7 from an unmeasured suspicion to a confirmed remediation
decision.
The [document-rooted match-path evidence](../Evidence/document-rooted-match-path-reevaluation-2026-08-31.md)
advances Finding 11 from hypothesis to an exact quadratic charged-work
mechanism; no optimized representation is selected.
The [global-frame cloning evidence](../Evidence/named-template-global-frame-cloning-2026-08-31.md)
confirms Finding 12's warm frame pressure while retaining prepared-XDM anatomy
as a separate unmeasured hypothesis.

## Review posture and limits

This review tries to falsify current guarantees. It does not treat a passing
fixture, an accepted ADR, or a private API label as evidence that the associated
mechanism is correct. Findings distinguish confirmed mechanisms from hypotheses
that still need measurement. The repository describes the implementation as a
private, staged standards slice, so unsupported language surface is not itself a
finding; silently producing the wrong result, misclassifying unsupported
behavior, escaping a stated resource boundary, or overstating evidence is.

Validation performed:

- `cargo test --workspace --all-features`: passed (378 engine tests with 10
  ignored, 10 native-workbench tests, and 3 worker tests).
- Python 3.14 `tomllib` parse of
  `corpus/overlays/xslt30/private-slice-v0.toml`: failed at line 1031 with
  `Cannot overwrite a value`.
- Initial and final worktree status were inspected. The review document is the
  only intended change.

### Finding 1: The selected-corpus overlay is invalid while the test suite reports green

- Severity: High
- Confidence: High
- Area: Corpus
- Evidence:
  - `corpus/overlays/xslt30/private-slice-v0.toml:1025-1031` assigns
    `rationale` twice in the same `[[case]]` table. A standards-compliant TOML
    parser rejects the file at line 1031. The preceding
    `conflict-resolution-1202a` table at lines 984-989 has no rationale, which
    strongly suggests the second value was attached to the wrong case.
  - `crates/fastxslt/src/runtime/xslt30_template_dispatch_tests.rs:190-204`
    embeds the overlay as a string and checks only whether the case-name text
    occurs anywhere. It never parses the record or validates its disposition.
  - `crates/fastxslt/src/runtime/xslt30_output_inventory_tests.rs:637-640`
    independently checks that the requested case name occurs somewhere and
    that `execution = "passed"` occurs somewhere. The two strings need not
    belong to the same case.
  - The full workspace test run passed despite the syntactically invalid
    overlay. This directly demonstrates a misleading green mechanism.
  - ADR-0006 requires duplicate/conflicting observations to fail visibly and
    separate selection from execution; the current string checks establish
    neither invariant.
- Why this may be wrong:
  - The file might be intended only as human-readable notes rather than a
    machine-readable ledger. That interpretation conflicts with its `.toml`
    extension, its `[[case]]` schema, the tests' reliance on it, and the SDD's
    description of ledger-accounted features.
- Reproduction or falsification path:
  - Run `py -c "import tomllib,pathlib; tomllib.loads(pathlib.Path(r'corpus/overlays/xslt30/private-slice-v0.toml').read_text())"`, then run
    `cargo test --workspace --all-features`. The first command fails and the
    second passes. To expose the cross-record bug, change one exercised case to
    `execution = "failed"`; output inventory tests can still find a different
    global `execution = "passed"`.
- Expected impact:
  - Selected-case, pass-count, and conformance evidence can drift from the
    machine-readable artifact without CI noticing. Invalid, missing, or
    contradictory dispositions may be presented as passing evidence.
- Suggested next experiment:
  - Add one throwaway validation test that parses the entire overlay into a
    typed record list, requires unique `(set_file, case_name)` identity and one
    rationale/disposition per selected case, and proves that changing a tested
    case's execution disposition makes the test fail.
- Decision interaction:
  - This is an implementation/evidence violation of
    [ADR-0006](../ADR/ADR-0006-verification-ledger-invariants.md) and the model
    incubated by
    [AR-0011](../Architectural%20Reviews/AR-0011-corpus-verification-ledger-classification-and-reporting.md).
    It does not require a new architecture decision to validate the existing
    artifact honestly.

### Finding 2: Location paths retain duplicate nodes instead of normalizing every step

- Severity: High
- Confidence: High
- Area: Semantic
- Evidence:
  - `crates/fastxslt/src/xpath/path_experiment.rs:507-583` concatenates each
    context node's step results into `next`.
  - Deduplication at lines 509 and 574-575 is conditional on
    `uses_descendant_or_self_axis()`. Parent, self, child, and descendant steps
    can also converge on the same node, but no general identity deduplication or
    document-order normalization follows the step.
  - `step_candidates` at lines 600-609 returns parent, self, descendant, or
    descendant-or-self candidates independently for every current node.
  - Counterexample: for `<r><a/><a/></r>`, `/r/a/..` produces the same `r`
    `NodeId` twice. Applying templates to that path can execute the `r` template
    twice. XPath path evaluation requires duplicate elimination and document
    order; see [XPath 3.1 path operator semantics](https://www.w3.org/TR/xpath-31/#id-path-operator).
- Why this may be wrong:
  - The private parser might reject `..` after a multi-node step. It does not:
    `PathStep::ParentNamed`/parent-axis handling is present, and pinned path
    cases exercise parent navigation. Even if one lexical spelling were
    excluded, descendant-to-parent convergence provides the same mechanism.
- Reproduction or falsification path:
  - Add a focused private test that evaluates `/r/a/..` over two `a` siblings
    and asserts exactly one result whose identity is `r`. Then drive the same
    selection through `xsl:apply-templates` and count template output nodes.
- Expected impact:
  - Wrong sequence cardinality, duplicate template execution, duplicate result
    construction, incorrect focus size/position, and distorted resource use.
- Suggested next experiment:
  - Differentially run all admitted two-step axis pairs over a small branching
    tree against Saxon, comparing ordered `NodeId`-equivalent paths and
    cardinality.
- Decision interaction:
  - This is a defect inside the staged semantic profile selected by
    [ADR-0007](../ADR/ADR-0007-staged-modern-standards-profile.md), not a request
    to widen the profile. Any normalization must preserve the representation
    ownership and accounting constraints in AR-0007/AR-0013.

### Finding 3: Temporary-tree `xsl:copy` is implemented as unconditional deep copy

- Severity: High
- Confidence: High
- Area: Semantic
- Evidence:
  - `crates/fastxslt/src/runtime/temporary_tree_executor.rs:26-37` detects any
    matched template whose entire body is one `Instruction::Copy` and bypasses
    normal instruction execution.
  - `copy_temporary_node` at lines 295-324 recursively copies every descendant,
    always emits no attributes, and never executes the compiled `xsl:copy`
    body.
  - The compiler retains constructed attributes and body instructions in
    `crates/fastxslt/src/compile/instruction_compiler/source_copy_compiler.rs:14-41`.
  - The source-tree implementation at
    `crates/fastxslt/src/runtime/golden_runtime_experiment.rs:689-719` performs
    the expected shallow element copy, materializes compiled attributes, and
    executes the body. The temporary path therefore has observably different
    semantics for the same compiled instruction.
  - Counterexample: a temporary `<outer><inner/></outer>` matched by a template
    containing only `<xsl:copy/>` should yield an empty `outer` element; the
    current path deep-copies `inner`.
- Why this may be wrong:
  - The special case may have been intentionally admitted only as a shorthand
    for deep copy in one corpus case. The compiler and SDD call the operation
    `xsl:copy`, and no validation restricts it to an equivalent body. A private
    shortcut cannot safely redefine an accepted XSLT instruction without an
    explicit unsupported diagnostic.
- Reproduction or falsification path:
  - Construct a global temporary tree with a nested child; apply templates to
    the outer element using `<xsl:copy/>`, then compare the serialized result to
    the same source-tree transform and to Saxon.
- Expected impact:
  - Silent wrong results, skipped body instructions and parameters, missing
    constructed attributes, and undercounted instruction work.
- Suggested next experiment:
  - Add a three-case parity matrix for empty `xsl:copy`, copy with one
    constructed attribute, and copy with `xsl:apply-templates`, executed once
    against a source tree and once against the equivalent temporary tree.
- Decision interaction:
  - The SDD states that source and temporary trees share ranking and focus
    contracts. This is a second semantic execution path without parity and
    conflicts with the architectural guardrail in AR-0007. Correcting it is
    within the existing contract; broadening temporary-tree surface is not.

### Finding 4: `xsl:strip-space elements="*"` silently violates `xml:space="preserve"`

- Severity: High
- Confidence: High
- Area: Semantic
- Evidence:
  - `crates/fastxslt/src/xdm/whitespace_view.rs:14-33` hides every
    whitespace-only text child of every element based solely on node kind and
    lexical value. It never inspects `xml:space` or its inherited value.
  - `crates/fastxslt/src/compile/golden_stylesheet_experiment.rs:129-140`
    accepts exact `xsl:strip-space elements="*"` and activates that view.
  - ADR-0012 lines 56-70 explicitly say `xml:space` semantics are not admitted,
    but runtime neither rejects an affected source nor preserves its text. It
    silently applies a broader, incorrect rule.
  - Under XSLT whitespace stripping, `xml:space="preserve"` participates in
    deciding whether whitespace is stripped; see
    [XSLT 3.0 stripping whitespace from a source tree](https://www.w3.org/TR/xslt-30/#strip).
- Why this may be wrong:
  - The project could define its private strip-all policy to exclude all sources
    containing `xml:space`. No admission check or diagnostic enforces that
    precondition, and XML input is otherwise accepted normally.
- Reproduction or falsification path:
  - Transform `<r xml:space="preserve">   </r>` with
    `<xsl:strip-space elements="*"/>` and a text-node/count observation.
    Compare against Saxon and against the same source without `xml:space`.
- Expected impact:
  - Data loss and wrong string values, template dispatch, focus sizes, copying,
    and serialized results for valid XML documents.
- Suggested next experiment:
  - Add a four-case differential test covering inherited `preserve`, nested
    `default`, no `xml:space`, and the XML namespace prefix spelled explicitly.
- Decision interaction:
  - [ADR-0012](../ADR/ADR-0012-invocation-owned-whitespace-visibility-view.md)
    deliberately defers the broader rule. The bounded choices are to reject
    affected inputs as unsupported or reopen AR-0016/ADR-0012 to implement the
    required semantics; silently stripping is consistent with neither choice.

### Finding 5: Copying a namespaced descendant can lose its required in-scope binding

- Severity: High
- Confidence: High
- Area: Semantic
- Evidence:
  - `crates/fastxslt/src/xml/quick_xml_experiment.rs:487-512` records only
    namespace declarations physically present on the current start tag.
  - `crates/fastxslt/src/runtime/golden_runtime_experiment.rs:1202-1241`
    constructs a copied result element with only
    `source.namespace_declarations(node)`. The shallow-copy path does the same
    at lines 689-719.
  - `crates/fastxslt/src/runtime/golden_runtime_experiment/serialization.rs:329-409`
    constructs result namespace scope from retained declarations and result
    ancestors. `element_prefix` at lines 539-559 returns `FXSR1002` if the
    expanded namespace has no retained binding.
  - Counterexample: selecting and copying only `p:item` from
    `<root xmlns:p="urn:u"><p:item/></root>` gives the child an expanded name
    in `urn:u` but no local namespace declaration and no copied result ancestor
    from which to inherit one.
- Why this may be wrong:
  - The XML reader may expose inherited namespace declarations in the element's
    attribute iterator. `resolve_namespace_declarations` explicitly iterates
    only lexical attributes on `BytesStart`, while namespace resolution is used
    separately for the expanded name, so that would require behavior not shown
    by this code.
- Reproduction or falsification path:
  - Copy the isolated namespaced child with both `xsl:copy-of select="."` and
    `xsl:copy`; assert a serializable element with an appropriate declaration,
    not `FXSR1002`. Repeat with default and prefixed namespaces.
- Expected impact:
  - Valid source subtrees can fail serialization or lose namespace fidelity
    when their declaring ancestor is not copied.
- Suggested next experiment:
  - Add a namespace-copy matrix that selects descendants away from declaring
    ancestors and compares expanded names plus serialized namespace fixup to a
    reference processor.
- Decision interaction:
  - This belongs to the XDM/XML-mechanics boundary reviewed by AR-0008 and the
    current serialization contract. Fixup must remain engine-owned rather than
    relying on parser-specific inherited-declaration behavior.

### Finding 6: Native handle registries permit unbounded process-memory retention

- Severity: High
- Confidence: High
- Area: FFI
- Evidence:
  - `crates/fastxslt-dotnet-workbench/src/lib.rs:28-49` stores engines, controls,
    and outcomes in process-global `HashMap`s with no count or retained-byte
    limit.
  - `insert_outcome` at lines 89-97 and creation/control exports allocate a new
    monotonically increasing handle and retain the value until an explicit
    release call. A buggy caller can omit release indefinitely.
  - Successful result bytes are capped at 1 MiB at lines 659-670, but failure
    envelopes from `engine_failure`/`insert_boundary_failure` bypass
    `MAX_OUTCOME_BYTES`; `encode_failure` at lines 207-234 grows a `Vec` before
    any aggregate boundary check.
  - `insert_created_engine` at lines 261-273 inserts an engine before inserting
    the creation outcome. Failure to allocate/insert the outcome leaves an
    engine with no handle delivered to the caller.
  - ADR-0008 promises bounded structured failure bytes, but does not supply a
    registry quota or abandonment policy.
- Why this may be wrong:
  - The managed `SafeHandle` wrappers release handles on normal paths. The C ABI
    is independently exported, finalization is nondeterministic, process death
    skips cleanup, and the review threat model includes buggy foreign callers;
    wrapper discipline is not a registry bound.
- Reproduction or falsification path:
  - In a sacrificial process, repeatedly create controls, failure outcomes, or
    engines without release while sampling registry cardinality and RSS. Verify
    whether any call is eventually rejected before memory growth becomes
    operationally material.
- Expected impact:
  - Trivial denial of service in the in-process ASP.NET lane and unattributable
    retained generation/outcome memory. One client can degrade unrelated
    callers sharing the process.
- Suggested next experiment:
  - Add test-only registry accounting and run a bounded 100,000-operation
    abandonment probe, reporting count, retained bytes, and whether a configured
    ceiling yields a structured failure.
- Decision interaction:
  - [ADR-0008](../ADR/ADR-0008-unsafe-native-dotnet-workbench-boundary.md)
    already requires bounded failure envelopes and lifecycle evidence. A
    process-wide registry quota/ownership policy changes operational behavior
    enough to require review against ADR-0008/ADR-0010 before admission.

### Finding 7: Template-candidate scans are not represented in work budgets

- Severity: High
- Confidence: High
- Area: Resource
- Evidence:
  - `crates/fastxslt/src/runtime/template_selector.rs:27-62`, `65-106`, and
    `109-131` linearly scan every compiled matched template for ordinary,
    next-match, and apply-imports selection.
  - Exact-name, node-kind, mode, priority, and many nonmatching pattern checks at
    lines 134-249 perform no `InvocationControl::charge`. Only patterns that
    traverse source structure charge XPath visits.
  - The caller charges one XSLT instruction per template application, not one
    unit per candidate considered. Cancellation is also observed at charge
    points, so a large simple-pattern scan can run between observations.
  - Work is structurally bounded by stylesheet parse limits, but the product of
    template count and selected node count is not represented by the advertised
    XSLT/XPath limits. The same issue exists in temporary selection at
    `temporary_tree_executor.rs:64-100`.
- Why this may be wrong:
  - Current XML event limits may keep stylesheets small enough for acceptable
    latency. That is an environmental cap, not semantic work accounting, and a
    repeated scan can multiply the capped template count by every source-node
    dispatch.
- Reproduction or falsification path:
  - Compile the largest admitted stylesheet containing mostly nonmatching exact
    element rules, apply templates over the largest admitted broad source, and
    compare elapsed time/cancellation latency with reported work counters.
- Expected impact:
  - Budget bypass in the sense that large CPU cost can complete under a small
    semantic-work count; delayed cooperative cancellation; avoidable
    `O(nodes × templates)` latency under adversarial but structurally valid
    input.
- Suggested next experiment:
  - Instrument only `candidates_considered` and cancellation-observation delay
    for a template-count/source-node sweep; do not add an index until the fanout
    and consumer-visible cost are measured.
- Decision interaction:
  - AR-0010 requires honest cooperative control and AR-0013 explicitly names
    template-selection fanout as a measurement target. An index is not yet
    justified, but missing work attribution is already a resource-contract gap.

### Finding 8: Temporary apply-templates loses sequence position and size

- Severity: Medium
- Confidence: High
- Area: Semantic
- Evidence:
  - `crates/fastxslt/src/runtime/temporary_tree_executor.rs:145-191` builds an
    ordered `selected` vector and invokes `apply_temporary_template` once per
    node without passing the offset or selected length.
  - `SequenceContext::for_temporary_template` in
    `golden_runtime_experiment.rs:419-429` inherits `focus_position = 1` and
    `focus_size = 1` from `SequenceContext::new`.
  - The source path correctly computes and passes `offset + 1` and
    `focus_size` at `golden_runtime_experiment.rs:885-899`.
  - Literal attribute value templates consume those fields through
    `result_tree.rs:123-124`, so multiple temporary nodes all observe
    `position() = 1` and `last() = 1`.
- Why this may be wrong:
  - The admitted temporary path might not claim focus AVTs. The same compiler
    accepts them in templates that can be selected for both source and
    temporary nodes, and runtime produces values rather than an unsupported
    diagnostic.
- Reproduction or falsification path:
  - Select two temporary sibling elements and emit
    `<seen p="{position()}" n="{last()}"/>`; expected values are `1/2` and
    `2/2`, while the current construction predicts `1/1` twice.
- Expected impact:
  - Wrong AVTs and any admitted behavior depending on focus position/size when
    templates process temporary sequences.
- Suggested next experiment:
  - Add one source-versus-temporary focus parity test over two elements and one
    intervening text node.
- Decision interaction:
  - This violates the SDD's current focus contract and the temporary/source
    parity stated around SDD sections 3.3-3.4. It should be repaired within the
    private seam, without creating a generalized provider abstraction.

### Finding 9: Forward global dependencies are accepted, then misreported as invalid input

- Severity: Medium
- Confidence: High
- Area: Semantic
- Evidence:
  - The compiler accepts a global `select="$name"` without resolving declaration
    order in `compile/golden_stylesheet_experiment.rs:297-378` and stores global
    bindings in top-level source order at lines 103-159.
  - `runtime/runtime_context.rs:116-200` materializes defaults in that stored
    order and resolves a variable reference only from values already inserted.
  - A forward reference falls through to `FXRT0002`, category `Invalid`, at
    lines 180-191. XSLT global dependencies are not defined by declaration
    order; cycles and dependencies require dependency analysis.
- Why this may be wrong:
  - Forward global dependencies may be outside the deliberately admitted
    expression slice. If so, compilation should identify them as unsupported
    rather than accept them and later classify a valid stylesheet as invalid.
- Reproduction or falsification path:
  - Declare `$first` with `select="$later"` before `$later`, then reverse the two
    declarations. The semantic result should not change merely because source
    order changes.
- Expected impact:
  - Valid stylesheets fail depending on declaration order, and hosts receive the
    wrong machine-readable failure category.
- Suggested next experiment:
  - Add a two-node global dependency graph test covering forward, backward, and
    cyclic references; compare result and diagnostic identity to Saxon.
- Decision interaction:
  - Dependency graph ownership is a compiler/runtime boundary question under
    ADR-0007 and AR-0001. If general dependency ordering remains deferred, an
    explicit compile-time unsupported boundary is consistent with current
    architecture.

### Finding 10: Concurrent cancellation writes can corrupt the worker protocol frame

- Severity: Medium
- Confidence: Medium
- Area: Concurrency
- Evidence:
  - `workbenches/FastXSLT.AspNet.Workbench/FastXsltWorkerClient.cs:273-305`
    holds `_gate` across controlled request start and completion; the gate is
    released only by `ReadControlledCompletionAsync` at lines 422-446.
  - Cancellation intentionally bypasses `_gate`. `SendCancellationAsync` at
    lines 449-455 performs separate asynchronous writes for opcode, length/data,
    and flush.
  - Multiple cancellation-producing calls can therefore write to the same
    `_input` concurrently. Byte writes are not composed into one atomic frame,
    and no dedicated write lock/channel serializes them.
- Why this may be wrong:
  - `ControlledTransformHandle.CancelAsync` may enforce one-shot cancellation
    for the normal public path, and production callers may never invoke the
    unrelated-cancellation experiment concurrently. The client nevertheless
    exposes multiple experimental cancellation paths, and stream writes do not
    promise frame-level atomicity.
- Reproduction or falsification path:
  - Use a yielding/fragmenting test stream or a long request identity; issue
    simultaneous cancellation sends while a controlled transform is active;
    assert that the worker reads two intact cancel frames and remains usable.
- Expected impact:
  - Protocol desynchronization, worker termination, miscorrelated cancellation,
    and failure of unrelated requests sharing that worker process.
- Suggested next experiment:
  - Inject a stream that yields after every byte and run 10,000 paired
    cancellation sends, validating the exact captured byte-frame sequence.
- Decision interaction:
  - AR-0010 and ADR-0011 require correlated bounded framing and honest hard-
    isolation behavior. Serializing outbound control frames is compatible with
    those decisions; changing cancellation ordering semantics would require
    review.

### Finding 11: Document-rooted match paths can reevaluate the whole path for every candidate

- Severity: Medium
- Confidence: High
- Area: Optimization
- Evidence:
  - `crates/fastxslt/src/runtime/template_selector.rs:475-505` evaluates a
    document-rooted `LocationPath` from the document node for each node whose
    pattern applicability is tested, then linearly calls `selected.contains`.
  - Ordinary selection already tests every template for every dispatched node.
    A broad apply-templates traversal plus one absolute path pattern can repeat
    the same document scan and allocation many times.
  - XPath node visits are charged, so this is not the same accounting omission
    as Finding 7; it is a credible `O(candidate nodes × path evaluation)` hot
    path that may exhaust budgets long before useful semantic work completes.
- Why this may be wrong:
  - Small admitted documents, early budget termination, or rare absolute
    patterns may make the cost immaterial in current workloads. No profile in
    the reviewed evidence attributes time to this exact loop.
- Reproduction or falsification path:
  - Sweep source width from 10 to the admitted maximum with one absolute
    multi-step match pattern and broad recursive dispatch. Record path
    evaluations, visited nodes, allocations, p50/p99 latency, and budget
    exhaustion point.
- Expected impact:
  - Superlinear warm latency and allocation churn in a standards-relevant
    dispatch path; poor scaling can be mistaken for generic XPath cost.
- Suggested next experiment:
  - Compare the current evaluator with a test-only per-invocation membership
    bitset computed once for that compiled path, including preparation cost and
    retained/peak memory.
- Decision interaction:
  - This is exactly the stylesheet-activated index/specialization question
    incubated by
    [AR-0013](../Architectural%20Reviews/AR-0013-prepared-representation-and-data-layout-audit.md).
    Measurement can proceed, but caching across snapshots/generations remains
    outside AR-0009's guardrails.

### Finding 12: Warm execution repeatedly clones global maps and per-node metadata

- Severity: Optimization
- Confidence: Medium
- Area: Optimization
- Evidence:
  - `runtime/runtime_context.rs:106-113` clones the complete global atomic
    `BTreeMap` for each new `RuntimeVariables` frame.
  - Named template calls do this again at
    `golden_runtime_experiment.rs:1390-1405`; template parameter binding also
    constructs fresh maps. Work therefore scales with total globals, not only
    bindings referenced or shadowed by the call.
  - `xdm/owned_tree_experiment.rs:32-44` stores multiple growable vectors,
    strings, and a `SourceLocation` in every node. `push_node` stores
    `resource.to_owned()` per node at lines 248-258. Expanded names and namespace
    strings are likewise occurrence-owned.
  - AR-0013 already identifies warm allocation churn, name duplication, and
    prepared-XDM byte anatomy as unmeasured high-probability areas; the code
    confirms the mechanisms but not their consumer-visible magnitude.
- Why this may be wrong:
  - Current global counts and documents may be tiny, clones may be cheap relative
    to semantic work, and interning/overlay frames can lose on lookup and
    locality. This is deliberately not classified as a correctness defect.
- Reproduction or falsification path:
  - Run a matrix over 0/16/64/256 globals and shallow named-template recursion,
    separately measuring Rust allocation count/bytes and latency. For XDM,
    report unique versus occurrence counts and retained bytes by field.
- Expected impact:
  - If the hypothesis holds, elevated warm allocation, cache pressure, prepared
    memory, and higher break-even reuse counts at exactly the compile-once/
    transform-many boundary the project targets.
- Suggested next experiment:
  - Use the existing allocation-observation seam to attribute one representative
    warm workload to global-frame clones versus result construction, changing no
    representation until that attribution is available.
- Decision interaction:
  - [AR-0013](../Architectural%20Reviews/AR-0013-prepared-representation-and-data-layout-audit.md)
    is the correct incubator. Any optimized form must preserve source identity,
    diagnostics, generation ownership, cancellation, and the safe reference
    path; no new ADR is justified by static inspection alone.

## 1. Top five correctness and security risks

1. **Corpus evidence can be green while its ledger is syntactically invalid.**
   This weakens every claim that depends on selected-case accounting, not just
   the two malformed records.
2. **XPath path results can contain duplicate nodes.** This can multiply
   template execution and corrupt sequence focus.
3. **Temporary-tree `xsl:copy` silently performs different semantics.** The
   shortcut deep-copies descendants and skips the compiled body/attributes.
4. **Whitespace stripping can delete `xml:space="preserve"` content.** Valid
   source data is lost without an unsupported diagnostic.
5. **The native C ABI has no registry memory ceiling.** A buggy or hostile
   in-process caller can retain engines, controls, and large outcomes until the
   process is exhausted.

The namespace-copy defect is close behind these five because it turns valid
namespaced subtrees into serialization failures. It should be handled in the
same initial correctness tranche.

## 2. Top five performance opportunities

These are experiment priorities, not implementation recommendations.

1. Measure template candidates considered per dispatch and test a compiled
   mode/kind/expanded-name candidate index only if fanout is material.
2. Measure repeated document-rooted path-pattern evaluation and compare it with
   an invocation-owned membership/index strategy.
3. Attribute warm allocations caused by cloning the complete global binding map
   into every template frame; compare with a parent/overlay frame.
4. Complete prepared-XDM byte anatomy, especially repeated resource identities,
   expanded names, namespace URIs, vector capacity, and per-node locations.
5. Attribute result construction, UTF-8 serialization, native outcome retention,
   P/Invoke copy, managed byte-array allocation, and final string decoding as
   separate end-to-end phases before proposing a sink or zero-copy boundary.

## 3. Top five architectural areas that are stronger than expected

1. **Authority is explicit.** Stylesheet dependencies flow through sealed
   snapshots and resolver decisions; engine execution does not silently acquire
   filesystem or network access. Dependency depth, count, and bytes are bounded.
2. **Compiled and invocation state are deliberately separated.** Prepared XDM
   is immutable, invocation controls and parameters are local, and generation
   replacement retains old state through leases instead of mutating it.
3. **Diagnostics are structured and generally preserve provenance.** Failure
   categories, codes, request identities, and source locations cross the Rust,
   worker, and managed boundaries without requiring display-string parsing.
4. **Unsafe Rust is tightly contained.** The ordinary engine forbids unsafe
   code; the native experiment confines it to documented byte-copy operations,
   validates scalar boundaries first, and releases registry locks before
   semantic work.
5. **The project records non-decisions unusually well.** ADRs and architectural
   reviews repeatedly distinguish private evidence from public conformance,
   performance, streaming, cache, and host-boundary guarantees. The main gaps
   found here are implementation/evidence drift from those records, not an
   absence of architectural intent.

## 4. Suspected false positives or areas the implementation already handles

- Process-global native quarantine after a caught panic initially looks like an
  availability defect. ADR-0008 explicitly chooses permanent quarantine rather
  than unsafe resurrection; the implementation matches that decision.
- A forged non-null foreign pointer can still crash or cause undefined behavior;
  `catch_unwind` cannot make arbitrary addresses safe. ADR-0008 explicitly puts
  readable/writable allocation validity on the caller. This is a trust-profile
  limitation, not evidence that `copy_input` exceeds its accepted unsafe scope.
- Pool `Dispose` races look hazardous when the pool types are read alone. The
  generation hosts retain leases and retire old pools only after lease drain on
  the intended path. Direct public workbench use still deserves a stress test,
  but the main generation lifecycle is stronger than a raw dispose scan implies.
- Worker completion order is not correlated by list position. Logical request
  identity is carried in framing and transform sets intentionally make no start
  or completion-order promise, consistent with ADR-0005.
- Import adapters read files in workbench/harness code, but the engine consumes
  qualified in-memory resources. This is consistent with ADR-0002 rather than a
  hidden ambient-I/O violation.
- Large runtime/compiler files cross ADR-0004 review triggers, but recent
  evidence shows named private-module extraction. Line count alone is not a
  cohesion violation; the temporary-tree parity issue is the stronger concrete
  boundary signal.

## 5. Missing evidence that blocks stronger claims

- No machine-parsed, schema-validated corpus ledger conserves inventory,
  selection, execution, assertion type, suite revision, overlay revision, and
  report identity end to end.
- No differential matrix covers admitted XPath axis composition, node identity,
  duplicate elimination, document order, position, and size.
- No source-versus-temporary semantic parity suite covers all shared
  instructions, ranking, built-in rules, continuation, parameters, and focus.
- No namespace fixup/copy matrix covers isolated descendants, default namespace,
  prefixed namespace, attribute namespace, redeclaration, and undeclaration.
- No whitespace suite covers inherited `xml:space`, nested `default`, parser
  normalization, and every source consumer behind the visibility view.
- No sustained native ABI abandonment test measures registry cardinality,
  retained bytes, failure-envelope size, finalizer delay, or process behavior
  under handle exhaustion.
- No adversarial worker framing test injects partial writes, concurrent control
  messages, truncated frames, delayed reads, process exit at every byte boundary,
  and recovery/replacement behavior.
- Resource counters are not calibrated against CPU time, allocations, retained
  bytes, or cancellation observation for template selection and serialization.
- Existing ASP.NET comparisons explicitly exclude Rust allocation attribution
  and do not establish total native retained memory, peak memory, sustained
  load, or result-transfer phase cost.
- The native unsafe seam still lacks the broader Miri/sanitizer/fault-injection
  evidence contemplated by ADR-0008; ordinary unit tests cannot validate foreign
  allocation contracts.

## 6. Recommended follow-up order

1. **Repair evidence integrity first.** Parse and validate the overlay, correct
   the duplicate/missing rationale, bind execution assertions to the same typed
   case record, and make the current misleading-green reproduction fail.
2. **Lock down small semantic counterexamples.** Add differential tests for
   `/r/a/..`, temporary empty `xsl:copy`, temporary `position()/last()`,
   `xml:space="preserve"`, isolated namespaced-child copy, and forward globals.
3. **Decide unsupported versus implemented boundaries.** For `xml:space` and
   forward globals, either reject before execution with structured
   `Unsupported` outcomes or reopen the relevant review and implement the full
   bounded semantics.
4. **Close native and worker denial paths.** Measure registry abandonment and
   failure-envelope size, then review a quota/ownership policy; independently
   serialize cancellation frames and stress partial/concurrent writes.
5. **Make work accounting honest.** Instrument template candidate comparisons,
   path reevaluations, allocation bytes, and maximum cancellation-observation
   delay. Revisit budget domains based on measured mechanisms.
6. **Run targeted performance experiments.** Only after semantic parity and
   accounting are stable, test dispatch indexes, path membership, overlay
   variable frames, XDM interning/layout, and result-transfer alternatives under
   the AR-0013 method.
7. **Regenerate evidence and claims.** Re-run the exact local gates, typed ledger
   report, differential corpus cases, native/worker adversarial tests, and
   end-to-end ASP.NET measurements before publishing a stronger standards,
   security, lifecycle, or performance statement.
