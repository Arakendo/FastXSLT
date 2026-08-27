# Runtime and Compiler Decomposition Review

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Checkpoint | `c2025fb` with all verification gates passing |
| Governing decision | ADR-0004 |
| Runtime before review | 2,431 physical lines |
| Runtime after preparatory test move | 1,862 physical lines |
| Runtime after first semantic extraction | 1,564 physical lines |
| Runtime after value-evaluation extraction | 1,310 physical lines |
| Extracted general test unit | 564 physical lines |
| Extracted transform-set unit | 304 physical lines |
| Extracted value-evaluation unit | 267 physical lines |
| Stylesheet compiler before extraction | 1,771 physical lines |
| Top-level compiler after extraction | 1,019 physical lines |
| Extracted instruction compiler | 775 physical lines |
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

## First semantic extraction

Private transform-set composition moved to `transform_set_experiment.rs`. The
unit owns request/result identities, request and result admission limits,
source-authority and admitted-resource checks, unordered execution correlation,
per-request control construction, source preparation, and final serialization
coordination for the test-only batch experiment.

The unit consumes the existing semantic entry points for principal-source,
initial-mode, and initial-template invocation. It does not evaluate
instructions, select templates, own XPath semantics, construct result nodes, or
implement serialization. The runtime parent imports only the private request,
policy, builder, and execution names needed by its child tests; production
workbench paths do not acquire the test-only batch surface.

This is a responsibility-coupling reduction rather than a line-only move. Batch
admission can change independently of sequence evaluation, while the invocation
engine remains the single semantic path used by batch execution.

## Second runtime semantic extraction

Dynamic `xsl:value-of` evaluation moved to `value_evaluator.rs`. The unit owns
dispatch across the admitted value-expression variants, their XPath evaluator
adapters, variable and source string-value conversion, separators, and the
translation of value-specific evaluator failures into existing runtime
diagnostics.

The sequence engine calls one private `execute_value_of` operation. The child
uses the invocation context, result nodes, text append boundary, and shared
failure constructors, but it does not call sequence execution, template
dispatch, serialization, transform-set composition, or host adapters. This
one-way dependency removes the XPath-family imports from the sequence owner and
allows value semantics to change without navigating template selection.

The runtime composition owner falls from 1,564 to 1,310 physical lines; the
value-evaluation owner is 267 lines.

## Required semantic seams

The first required seam is now extracted. Continue by reviewing these
independently demonstrated seams:

1. invocation setup, globals, parameters, and temporary-tree materialization;
2. remaining instruction control flow and result construction;
3. template selection, modes, pattern matching, and named-call depth; and
4. structured failure construction and control-failure translation.

## Compiler extraction

The private `instruction_compiler.rs` now owns sequence-constructor traversal,
instruction lowering, literal-result namespace collection, instruction-local
attribute and content validation, and the expression-specific parsing needed
by the admitted instructions. It consumes stylesheet XDM and returns existing
private semantic instructions.

The parent retains stylesheet-root and top-level declaration assembly,
templates and globals, cross-template reference validation, shared structural
checks, and structured compilation failures. It calls three explicit child
operations: sequence compilation, template-mode parsing, and literal-result
namespace collection. Neither module exposes the semantic IR publicly, imports
runtime/host policy, or creates an alternate compilation path.

This reduces the compiler composition owner from 1,771 to 1,019 physical lines;
the instruction owner is 775 lines. Expansion of an admitted instruction or
its expression parser no longer requires navigating top-level assembly and
cross-template validation.

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

Both structural moves preserve all 56 focused runtime tests. Full repository
verification remains the conservation gate before the semantic extraction is
committed.

## Disposition and reopening

Two runtime semantic extractions and the compiler instruction extraction are
complete. Reassess the remaining 1,310-line invocation engine before adding
another semantic family that touches template dispatch, temporary trees, or
sequence control flow. Retain the 1,019-line top-level compiler at this
checkpoint; reopen it at 1,200 lines or when a new top-level declaration or
validation phase demonstrates another owner. Reopen the 775-line instruction
compiler if instruction lowering and expression parsing develop independently
pressured subsystems or it crosses 1,000 lines.

The campaign is complete only when named modules reduce responsibility
coupling; line count below a threshold alone does not close this review.
