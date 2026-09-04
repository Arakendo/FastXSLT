# Instruction Boolean-Compiler Decomposition

| Field | Value |
| --- | --- |
| Date | 2026-09-03 |
| Trigger | Finding 7 of the [second adversarial review](../Reviews/adversarial-engine-review-2026-09-03.md) |
| Governing decision | [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md) |
| Disposition | Required structural checkpoint independently verified; Finding 7 closed |

## Change

Boolean-expression recognition and compilation moved from
`compile/instruction_compiler.rs` to the private
`compile/instruction_compiler/boolean_expression_compiler.rs` child. The parent
now chooses when an `xsl:if` or `xsl:when` test requires boolean compilation and
calls one typed operation with the expression, source location, and existing
string-comparison policy.

The child owns boolean-expression recursion, lexical recognition, and lowering
to the existing private `BooleanExpression` representation. It may call the
existing typed conditional-expression and path parsers and pure static lexical
helpers. It does not traverse sequence constructors, compile other
instructions, evaluate runtime state, acquire resources, or own host policy.

## Physical checkpoint

At the auditor's `b75e4e7` checkpoint,
`compile/instruction_compiler.rs` contained 2,237 physical lines. After this
extraction and formatting it contains 1,817 lines; the new private boolean
compiler contains 443 lines. This is a responsibility move rather than a claim
that total source volume decreased.

Control flow remains one-way: the sequence-constructor owner selects the
boolean compiler, and the boolean compiler returns the existing typed result or
existing structured `CompileFailure`. The child imports stateless lexical and
diagnostic helpers from its parent plus existing typed compiler parsers; it does
not call back into sequence-constructor traversal. No trait, callback, dynamic
dispatch, allocation policy, public Rust API, native ABI, or second execution
path was introduced.

## Conservation

The extraction intentionally preserved the original recognizer ordering,
branch structure, diagnostics, locations, and typed outputs. It did not add an
expression family or change a corpus disposition.

Validation after the move:

- `cargo test -p fastxslt compile::golden_stylesheet_experiment --all-features`:
  passed; 52 passed and 1 ignored measurement probe;
- `cargo test --workspace --all-features`: passed; 622 engine tests passed with
  17 ignored, 18 native tests passed with 2 ignored, and 4 worker tests passed;
- `cargo fmt --all --check`: covered by the final repository verification gate.

## Independent closure re-audit

Independent review at `e8115d7` compared the extraction with the pre-move
recognizer, traced both parent call sites, inspected the child's imports and
responsibility boundary, confirmed the 1,817/443 physical line counts, and ran
the focused compiler suite plus `scripts/verify.ps1`. The focused suite passed
52 tests with one ignored manual measurement probe. The full verification gate
passed formatting, Clippy, 622 engine tests, 18 native tests, 4 worker tests,
documentation, Markdown links, and pinned conformance-source checks.

Finding 7's immediate reopening condition is therefore independently verified
closed. The broader ADR-0004 campaign remains active for the serializer
HTML-profile recognizer, runtime boolean evaluator, runtime source-copy seam,
and final coupling and line-count review; those planned checkpoints are not a
continuation of this adversarial finding.
