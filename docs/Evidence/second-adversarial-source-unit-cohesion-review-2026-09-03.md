# Second Adversarial Source-Unit Cohesion Review

| Field | Value |
| --- | --- |
| Date | 2026-09-03 |
| Semantic checkpoint | `32dd322`, all repository gates passing |
| Governing decision | [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md) |
| Review trigger | Finding 7 of the [second adversarial review](../Reviews/adversarial-engine-review-2026-09-03.md) |
| Disposition | Review obligation discharged; checkpointed decomposition campaign active |

## Current inventory

Physical line counts were measured after the first behavior-preserving test
extraction:

| Unit | Lines | Current responsibility | Disposition |
| --- | ---: | --- | --- |
| `compile/golden_stylesheet_experiment.rs` | 1,783 | Top-level stylesheet assembly, globals, templates, character maps, common static diagnostics | Retain composition owner at this checkpoint; tests extracted |
| `compile/golden_stylesheet_tests.rs` | 1,214 | Compiler composition invariants and one ignored scaling measurement | Retain as a cohesive private test owner; inspect on substantive growth |
| `compile/instruction_compiler.rs` | 1,851 | Sequence-constructor lowering plus selection among instruction-local XPath/boolean compilers | Decompose before another expression-family expansion |
| `runtime/golden_runtime_experiment.rs` | 2,833 | Invocation orchestration, sequence execution, template recursion, source copying, boolean evaluation, result construction | Active decomposition debt; use separate structural checkpoints |
| `runtime/golden_runtime_experiment/serialization.rs` | 2,060 | Method selection, bounded HTML-profile validation, XML/XHTML/HTML/text serialization, escaping, namespace fixup, output budgets | Extract bounded HTML-profile recognition before more output semantics |

The compiler, runtime, and serializer crossed ADR-0004 numeric or retained
reopening triggers. The instruction compiler crossed its earlier 1,000-line
reopening trigger. Repeated standards campaigns also satisfy the independent
responsibility-change signal. A blanket retain decision is therefore not
supported.

## Completed extraction

The 1,214-line compiler composition test body moved unchanged from
`golden_stylesheet_experiment.rs` to the private
`golden_stylesheet_tests.rs`. The parent retains only the test module
declaration. All 40 ordinary compiler tests and the ignored character-map
measurement keep the same Rust module path and private access.

This is a navigation and invariant-ownership improvement rather than a claim
that test dependencies became narrow. The tests intentionally exercise the
whole compiler composition boundary. It nevertheless removes test mechanics
from a production owner and brings that owner below the mandatory 2,001-line
threshold without moving semantics.

## Retained compiler composition owner

The top-level compiler remains the only owner that assembles one stylesheet
program from top-level declarations. It calls private instruction, output,
module, mode, pattern, and validation children. It owns no runtime evaluation,
serialization, host policy, filesystem authority, or public API.

Its remaining globals/templates/character-map responsibilities are coupled by
top-level declaration order, import precedence, shared validation, and final
program assembly. Splitting them during the just-completed correctness and
character-map optimization tranche would weaken attribution. Reopen before a
new top-level declaration family, a second character-map representation, or at
2,001 lines, whichever occurs first.

## Required instruction-compiler seam

The instruction compiler still combines sequence-constructor traversal with a
large boolean-expression recognizer. Template invocation, computed attributes,
literal attributes, source copy, and conditional value expressions already
have private children. The next structural checkpoint should move boolean
expression compilation and its lexical helpers behind one private typed
operation.

The child may consume the stylesheet document, expression, source location,
namespace lookup, and existing typed path parser. It must not traverse sequence
constructors, compile unrelated instructions, call runtime code, or own host
policy. The parent remains responsible for instruction dispatch and choosing
which expression family is attempted.

### Completed checkpoint

The required boolean-expression seam was extracted after the closure re-audit
reopened Finding 7 for bypassing this ordering constraint. At the auditor's
`b75e4e7` checkpoint, `instruction_compiler.rs` contained 2,237 lines. The
behavior-preserving move leaves 1,817 lines in the parent and places 443 lines
in the private `boolean_expression_compiler.rs` child.

The child consumes an expression, source location, and existing comparison
policy and returns the existing typed boolean expression or structured compile
failure. It owns boolean lexical recognition and recursive lowering, depends
only on existing pure static compiler helpers, and has no constructor traversal,
runtime, resource, host-policy, public-API, or ABI responsibility. The full
workspace suite passes unchanged. See the
[instruction boolean-compiler decomposition evidence](instruction-boolean-compiler-decomposition-2026-09-03.md).

## Required runtime seams

The runtime parent remains the mutually recursive composition root for sequence
execution and template application. That recursion is a real reason not to
split it into callbacks or a broad mutable executor object merely to reduce line
count. Two responsibilities are nevertheless independently extractable:

1. Boolean evaluation can consume the invocation inputs, compiled boolean
   expression, context, variables, and control, returning a boolean or the
   existing structured failure. It must not execute instruction sequences or
   choose templates.
2. Source-node copying can consume source nodes and construct result nodes under
   the existing budget/control contract. It must not perform template
   selection, serialization, or resource acquisition.

Perform these as separate behavior-preserving commits. Reassess the remaining
sequence/template recursion after both extractions; do not manufacture a public
executor trait or second backend.

## Required serializer seam

The serializer now contains a substantial bounded HTML capability recognizer in
addition to output generation. Move HTML result-shape admission and version/mode
selection to a private child that reads immutable result nodes and output
settings and returns the existing private mode or failure. Serialization stays
the sole owner of emitted bytes, escaping, namespace fixup, indentation, output
budgets, and cancellation.

This dependency must remain one-way: serialization asks the HTML-profile child
whether the semantic result belongs to an admitted profile; the child never
writes output or mutates the result. Character-map composition remains compiler
owned, while character-map lookup remains serializer owned.

## Consequences and conservation

All work stays within the `fastxslt` crate and private visibility. No public Rust
API, native ABI, dependency, resource authority, snapshot identity, XDM node
identity/order, diagnostic code/location, unsafe surface, or corpus disposition
may change during these structural checkpoints. The moves must preserve:

- the 601-test semantic checkpoint, including all unchanged QT3/XSLT30 cases;
- direct, workbench, and isolated-worker lifecycle behavior;
- cancellation and every work/byte charge point;
- character-map release measurements and output semantics;
- prepared-engine retention accounting; and
- compilation and documentation gates.

Private child calls add no allocation or dynamic dispatch. Compile-time effects
are not currently established as a pressure source; clean/incremental build
changes remain observations rather than a decomposition goal.

## Reopening and completion

Finding 7's missing-review obligation is discharged by this inventory and
disposition. The decomposition debt is not declared complete. The roadmap must
retain the remaining serializer HTML-profile extraction, runtime boolean
evaluator extraction, runtime source-copy extraction, and a post-campaign
coupling/line-count review. The instruction boolean-compiler checkpoint is now
complete; its delayed completion and the auditor's reopening remain recorded
rather than rewritten as if the ordering constraint had been conserved.

If any extraction requires a broad shared mutable context, cyclic sibling
imports, changed diagnostics, or altered hot-path behavior, stop and record the
failed seam rather than calling physical fragmentation success.
