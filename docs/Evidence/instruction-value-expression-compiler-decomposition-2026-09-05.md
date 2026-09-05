# Instruction Value-Expression Compiler Decomposition

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `e69a742` |
| Governing decision | [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md) |
| Disposition | Behavior-preserving private extraction complete; semantic expansion may resume |

## Trigger and responsibility inventory

After the independently verified boolean-compiler extraction,
`compile/instruction_compiler.rs` contained 1,817 lines. Later standards work
added multiple value-expression families and compile-selected XPath 1.0
compatibility rules. At source checkpoint `e69a742`, the parent contained 2,526
physical lines and crossed ADR-0004's retained-review threshold again.

The unit also satisfied the responsibility trigger: sequence-constructor
traversal and instruction lowering shared a file with an independently testable
typed value-expression selection pipeline. The earlier cohesion review had
specifically required decomposition before another expression-family
expansion.

## Extracted boundary

`compile/instruction_compiler/value_expression_compiler.rs` now owns:

- selection and lowering of typed `xsl:value-of` expression families;
- the private stylesheet-version compatibility context used only during that
  selection;
- compilation of value-function and value-path forms;
- static literal, numeric, boolean, root, identity, and variable compositions;
  and
- mapping parser failures into the existing structured compile diagnostics.

Its inputs are the immutable stylesheet XDM document, instruction node,
expression text, and retained source location. Its output is the existing
private `ValueExpression` or existing `CompileFailure`. It imports only
stateless compiler vocabulary and typed parsers already visible to the parent.

The child explicitly does not own sequence-constructor traversal, instruction
ordering, template invocation, result construction, runtime evaluation,
resource acquisition, host policy, public API, or an alternate backend. The
dependency direction is parent sequence compiler to private value compiler;
the child does not call back into sequence traversal or receive a broad mutable
compiler context.

## Physical and coupling result

After formatting:

| Unit | Before | After |
| --- | ---: | ---: |
| `instruction_compiler.rs` | 2,526 | 1,451 |
| `instruction_compiler/value_expression_compiler.rs` | absent | 1,100 |

The new child crosses ADR-0004's 1,000-line inspection signal. It is retained as
one unit because its ordered expression-family selection and private helpers
share one typed input/output contract and one diagnostic policy. It contains no
tests, runtime state, host boundary, reference/optimized dual path, or second
language phase. Reopen it if another independently testable compiler phase
appears, ordinary changes touch distant unrelated regions, or it crosses 2,000
lines.

No public visibility, crate boundary, trait, callback, dynamic dispatch, plan
representation, dependency, native ABI, unsafe surface, allocation policy, or
runtime hot-path call was introduced.

## Conservation

The move preserved function bodies, recognizer order, typed outputs,
diagnostic codes/details/locations, and the shared lexical helper used by the
boolean and conditional compiler children. It intentionally changed no
semantic or corpus disposition.

The complete local OASIS XSLT 1.0 measurement after extraction reproduced the
checkpoint exactly:

- 1,061 initialized and 2,074 initialization failures;
- 860 executed successfully and 201 execution failures;
- 777 XML comparison passes and 50 mismatches; and
- all infrastructure and expected-error accounting remained in the same
  measured denominator.

The repository verification gate covers formatting, strict Clippy, workspace
tests, documentation, Markdown links, immutable submodules, and conformance
inventory. This record is a structural checkpoint, not new conformance or
performance evidence.
