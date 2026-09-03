# Instruction-Compiler Conditional-Expression Decomposition Review

Date: 2026-09-03

## Trigger

The completed XSLT30 `insn/choose` campaign raised
`compile/instruction_compiler.rs` to 2,050 lines. This crossed ADR-0004's
mandatory 2,000-line review threshold. Conditional-expression recognition had
also become an independently named responsibility: it owned balanced lexical
branch splitting, recursive conditional structure, typed path operands,
schema-prefix validation, and lowering into two narrow owned plans.

## Decision

Extract `instruction_compiler/conditional_expression_compiler.rs` as a private,
one-way child. The child owns only the two admitted XPath conditional forms and
returns an optional owned value expression or structured compilation failure.
The parent retains sequence-constructor traversal, instruction dispatch, and
selection among XPath expression families.

The parent is 1,813 lines after extraction and the child is 255 lines. A small
deep-equal diagnostic-mapping helper was also extracted within the parent to
keep the value-expression dispatcher below its focused function-size gate.

The child receives the stylesheet document, instruction node, expression, and
source location rather than a broad compiler context. It reads existing
namespace/path helpers from its parent and does not call sibling compiler
modules. Dependency direction remains parent to private child.

## Conservation

This extraction intentionally changes no language result. Conservation is
provided by:

- all 55 unchanged XSLT30 choose cases receiving the same passed,
  engine-unsupported, or expected-error disposition;
- the lazy division-by-zero branch cases `choose-1903` and `choose-1904`;
- focused alternate and rebound XML Schema prefix controls;
- complete workspace tests, documentation, corpus integrity, formatting,
  strict Clippy, and unsafe-surface verification.

## Claim boundary

This is private source-unit decomposition under ADR-0004. It creates no public
XPath compiler interface, general conditional grammar, alternate execution
backend, or new representation contract. Reopen if the child begins consuming
most parent state, gains non-conditional expression ownership, or either source
unit crosses another ADR-0004 review trigger.
