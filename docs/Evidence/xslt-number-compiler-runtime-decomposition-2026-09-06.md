# XSLT Number Compiler and Runtime Decomposition

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Trigger | Cohesive `xsl:number` compilation and execution grew while the OASIS XSLT 1.0 campaign added all three numbering levels |
| Governing decision | [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md) |
| Disposition | Private one-way extraction complete; corpus dispositions conserved |

## Change

The admitted `xsl:number` compiler moved from
`compile/instruction_compiler.rs` into the private
`compile/instruction_compiler/number_compiler.rs` child. The child owns the
instruction's static attributes, number-level selection, admitted count/from
pattern compilation, decimal-format compilation, and bounded value-expression
classification. The parent still owns sequence-constructor traversal and only
dispatches an `xsl:number` element to the typed child.

The matching executor moved from `runtime/golden_runtime_experiment.rs` into
the private `runtime/number_executor.rs` child. It owns numeric conversion,
single/multiple/any traversal, pattern matching, decimal-token formatting, and
the numbering-specific work charges. The parent retains invocation setup,
general instruction dispatch, shared source-context access, result text
construction, and control-failure translation.

Neither extraction changes the private semantic representations or introduces
a trait, callback, cache, alternate backend, public API, host policy, or unsafe
code.

## Physical checkpoint

After formatting:

| Unit | Physical lines |
| --- | ---: |
| `compile/instruction_compiler.rs` | 1,544 |
| `compile/instruction_compiler/number_compiler.rs` | 165 |
| `runtime/golden_runtime_experiment.rs` | 3,303 |
| `runtime/number_executor.rs` | 367 |

This is a responsibility move, not a claim that total source volume fell. The
runtime parent remains under broader ADR-0004 pressure, but numbering no longer
contributes a second semantic subsystem inside it.

Control flow remains one-way. Each parent selects its numbering child and the
child returns the existing typed instruction or execution result. The children
reuse narrowly scoped static helpers or runtime services from their parents;
they do not call general constructor traversal or instruction dispatch.

## Conservation

The extraction preserved numbering recognizer order, source locations,
structured diagnostics, XDM node identity, result construction, cancellation,
work budgets, and all existing typed values. The unchanged local OASIS XSLT
1.0 measurement reported:

- 3,173 catalog cases;
- 1,213 initialized;
- 1,002 executed successfully;
- 893 expected-result XML matches;
- 75 visible comparison mismatches, including seven carrying upstream doubts;
- 211 visible execution failures.

Focused `xslt10_number` verification passed before the conserved full corpus
measurement. Strict crate Clippy and formatting also passed after the move.

