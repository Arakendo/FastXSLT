# XSLT30 `conflict-resolution-1301` Positional Focus and ISO-8859-1 Bytes

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

The third behavior cannot be represented honestly by the UTF-8 Rust `String`
result lane and therefore requires a separate byte result.

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

## Pinned-case result

The unmodified upstream source and stylesheet now produce the native expected
XML through a bounded byte serializer. The result begins with the exact
ISO-8859-1 declaration and retains the four asserted blocks with positions
`2`, `4`, `6`, and `8`, size `9`, and the expected black/blue last-member
distinction. The upstream file-backed `assert-xml` is loaded without copying it
into first-party MIT-licensed fixtures.

This moves the complete apply-templates denominator to 49 selected passes and
one visible schema-aware not-run case.

## Conservation boundary

The existing UTF-8 `String` lane still rejects ISO-8859-1 rather than returning
mislabeled text. The byte lane admits only UTF-8 generally and the ASCII subset
of ISO-8859-1 required by this case. A negative test rejects non-ASCII result
characters with structured `FXSR1006 / unsupported`; it neither replaces them
nor claims general character-reference or Latin-1 encoding support. Byte limits
and `SerializedByte` work charges include the encoding declaration and body.

The evidence admits only the two exact positional pattern shapes and the two
exact whole-value focus AVTs. It does not admit general numeric predicates,
focus functions in XPath expressions, composite AVTs, sorting, schema-aware
matching, or a supported public byte serialization API.
