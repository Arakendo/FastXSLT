# Attribute-Set Compiler Decomposition -- 2026-09-16

Date: 2026-09-16  
Status: Verified structural evidence  
Decision basis: ADR-0004 source-unit cohesion and decomposition policy

## Pressure

Adding graph validation and recursive compile-time expansion for inherited
attribute sets raised `compile/instruction_compiler.rs` to 2,431 physical lines.
The new work had a named responsibility distinct from general sequence
constructor traversal: declaration validation, dependency-graph validation,
cycle/undefined-reference diagnostics, precedence ordering, and static
attribute expansion.

## Extraction

That responsibility now lives in the private typed child
`compile/instruction_compiler/attribute_set_compiler.rs`.

| Source unit | Before | After |
| --- | ---: | ---: |
| `instruction_compiler.rs` | 2,431 | 2,216 |
| `attribute_set_compiler.rs` | absent | 256 |

The child accepts a stylesheet document and node identities, returns ordinary
compiled attributes or structured compile failures, and calls the existing
computed-attribute compiler for the already-admitted static value forms. It
owns no runtime state, resource authority, host policy, public API, cache, or
alternate execution path. The parent retains sequence traversal and invokes
the child through two narrow validation functions plus one expansion function.

## Conservation

Focused tests preserve inheritance and override order and the exact undefined
and circular diagnostics. The complete repository gate passes with 951 engine
tests, 18 native tests, 18 worker tests, documentation, Markdown links, and
pinned-corpus integrity.

The parent remains above ADR-0004's 2,000-line review threshold. This extraction
removes the newly independent graph responsibility but does not declare the
remaining parent exempt from review. Another semantic family added directly to
the parent must first identify and review its ownership seam; line count alone
does not justify arbitrary splitting.
