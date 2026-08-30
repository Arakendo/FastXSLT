# XSLT30 `conflict-resolution-1301` Positional-Focus Prerequisite

Date: 2026-08-30

## Pressure

The remaining non-schema-aware case in the complete 50-case XSLT30
apply-templates denominator is `conflict-resolution-1301`. Its stylesheet
requires three independently observable behaviors:

- positional template predicates distinguish the last named `member` sibling
  from earlier matching siblings;
- matched templates observe their position and size in the full sequence
  selected by `xsl:apply-templates`, including whitespace text nodes;
- serialization requests ISO-8859-1 bytes.

The third behavior cannot be represented honestly by the current UTF-8 Rust
`String` result lane and remains separate.

## Executable prerequisite

A focused source supplies four `member` elements interleaved with five
whitespace text nodes. Default `xsl:apply-templates` therefore establishes a
nine-node focus sequence. Typed positional patterns select the first three
members with `position() &lt; last()` over the matching-member sibling sequence
and the fourth with `position() = last()`.

The selected templates construct attributes through the exact whole-value AVT
forms `{position()}` and `{last()}`. They observe full invocation positions
`2`, `4`, `6`, and `8`, with size `9`. A text-node rule suppresses result text
without changing the selected focus sequence.

Runtime continuation now preserves those focus values rather than resetting
them when a matched rule enters `xsl:next-match` or `xsl:apply-imports`. Built-in
child traversal establishes a new focus using the child's actual position and
the complete child-sequence size.

## Conservation boundary

This change does not select or pass `conflict-resolution-1301`. The upstream
stylesheet still requests `encoding="ISO-8859-1"`, which compilation rejects as
unsupported in the current string serialization lane. FastXSLT does not emit a
UTF-8 `String` whose XML declaration falsely names another encoding.

The evidence admits only the two exact positional pattern shapes and the two
exact whole-value focus AVTs. It does not admit general numeric predicates,
focus functions in XPath expressions, composite AVTs, sorting, schema-aware
matching, or a byte serialization API.
