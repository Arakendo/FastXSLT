# Runtime and Compiler Decomposition Review

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Checkpoint | `c2025fb` with all verification gates passing |
| Governing decision | ADR-0004 |
| Runtime before review | 2,431 physical lines |
| Runtime after preparatory test move | 1,862 physical lines |
| Extracted general test unit | 564 physical lines |
| Stylesheet compiler | 1,771 physical lines |
| Disposition | Active checkpointed decomposition required |

## Trigger and correction

`golden_runtime_experiment.rs` crossed ADR-0004's 2,001-line mandatory review
threshold. It also crossed every applicable reopening trigger retained by the
2026-08-25 runtime cohesion review: the whole unit exceeded 1,200 lines, its
production portion exceeded 700 lines, modes and temporary trees entered the
runtime, and independently selected standards campaigns repeatedly extended
distant regions.

The stylesheet compiler is in the 1,001-2,000 inspection band. Its growth is
not yet a numeric violation, but instruction compilation, expression-specific
parsing, top-level declaration handling, reference validation, and tree-access
mechanics are independently changing responsibilities. Its next substantive
semantic expansion therefore requires a named extraction or a renewed retain
decision; silent continued accumulation is not accepted.

## Responsibility inventory

The runtime currently owns:

- transform-set admission, identity conservation, resource checks, and result
  correlation for the private batch experiment;
- stylesheet import/parse/XDM/compile composition;
- invocation entry and parameter binding;
- global-value and temporary-tree materialization;
- instruction-sequence evaluation and result construction;
- template selection, named calls, modes, patterns, and built-in rules;
- XPath-family adapters and work accounting; and
- structured runtime failure translation.

Serialization already has a private owner. Control-phase, host-workflow, and
individual W3C denominator tests already use named private units. The remaining
inline general contract tests mixed implementation navigation with roughly 570
lines of fixtures and assertions.

The stylesheet compiler currently owns:

- stylesheet-root and top-level declaration processing;
- output settings, globals, matched/named templates, and modes;
- instruction and literal-result-element compilation;
- several expression-specific parsers and validators;
- named-template reference validation; and
- XML tree access, QName/NCName checks, diagnostics, and failure mapping.

## Preparatory extraction

The general runtime contract tests moved unchanged to
`golden_runtime_tests.rs`, a private test unit named for the invariant boundary
it exercises. The parent retains the test module declaration, so test names and
access to private runtime contracts are conserved.

This move improves navigation and separates executable tests from production
semantics, but it does not reduce semantic coupling: those tests intentionally
exercise much of the private composition owner. It is therefore a preparatory
checkpoint, not sufficient evidence that the runtime decomposition is complete.

## Required semantic seams

The next behavior-preserving extraction should separate private transform-set
composition from invocation semantics. That unit should own request/result
identity, admission limits, source-authority validation, unordered result
correlation, and the test-only scheduling loop. It may call the semantic
invocation entry points but must not own instruction evaluation, template
selection, or result serialization.

After that extraction, review these independently demonstrated seams:

1. invocation setup, globals, parameters, and temporary-tree materialization;
2. instruction evaluation and result construction;
3. template selection, modes, pattern matching, and named-call depth; and
4. structured failure construction and control-failure translation.

For the compiler, the leading candidate is a private instruction compiler that
consumes stylesheet XDM plus static context and returns existing semantic
instructions. Top-level assembly and cross-template validation remain in the
composition owner. The extraction must not expose the private semantic IR.

## Dependency and conservation requirements

Dependency direction remains inward: batch composition calls one invocation
semantic path; invocation evaluation calls XPath owners and constructs semantic
results; serialization consumes results only. Compiler assembly may call a
private instruction compiler, but the instruction compiler must not import
runtime or host policy.

No phase may change public Rust APIs, native ABI, resource authority, snapshot
identity, XDM identity/order, evaluation order, template selection, diagnostic
codes/locations, cancellation or budget charging, result correlation,
serialization, unsafe surface, or corpus classification. Structural commits
remain separate from semantic fixes and optimizations.

The preparatory move preserves all 56 focused runtime tests. Full repository
verification remains the conservation gate before the move is committed.

## Disposition and reopening

Pause further denominator-driven growth in these two units until the first
runtime semantic extraction is complete. Resume standards work after the same
focused and repository-wide gates pass across that checkpoint. Reassess the
compiler before its next semantic family or at 2,000 physical lines, whichever
comes first.

The campaign is complete only when named modules reduce responsibility
coupling; line count below a threshold alone does not close this review.
