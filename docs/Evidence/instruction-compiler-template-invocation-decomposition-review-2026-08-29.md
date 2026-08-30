# Instruction-Compiler Template-Invocation Decomposition Review

Date: 2026-08-29

## Trigger

Adding the bounded computed-attribute plan raised
`compile/instruction_compiler.rs` to 1,081 lines. Template invocation was also
an independently named responsibility: it owned apply/call instruction shapes,
parameter arguments, selection parsing, mode parsing, and their diagnostics.
The file therefore met ADR-0004's calibration trigger of more than 1,000 lines
plus a responsibility boundary.

## Decision

Extract two private, one-way children:

- `computed_attribute_compiler.rs` owns leading computed-attribute validation
  and specialization; and
- `template_invocation_compiler.rs` owns `xsl:apply-templates`,
  `xsl:apply-imports`, `xsl:call-template`, with-parameter arguments, apply
  selections, and mode lexicals.

The parent retains sequence-constructor traversal, instruction dispatch,
literal result composition, text/value/variable/conditional construction, and
the small delegation points used by that traversal. It is 854 lines after
extraction. The invocation child is 289 lines and the computed-attribute child
is 125 lines.

The child does not receive a broad compiler context. It consumes the stylesheet
document and the exact declaration node, calls existing static-context/path
helpers, and returns owned semantic instructions or structured compilation
failures. Dependency direction remains parent to private child.

## Conservation

The extraction intentionally changes no language result. Conservation is
provided by the complete workspace suite, including:

- all previously admitted apply-templates, next-match, apply-imports, named
  template, parameter, mode, and path cases;
- XSLT30 `include-0301` import-precedence and repeated continuation behavior;
- XSLT30 `include-0202` imported parameter binding and computed attribute;
- structured invalid/unsupported diagnostics and source locations;
- semantic-inspection feature counts; and
- formatting, strict Clippy, documentation, corpus, and unsafe-surface gates.

## Claim boundary

This is a private source-unit decomposition under ADR-0004. It creates no crate,
public compiler API, alternate semantic path, host authority, cache, or new
standards claim. Reopen if the parent again crosses a calibration trigger or if
the child begins consuming most parent state instead of owning invocation
compilation coherently.
