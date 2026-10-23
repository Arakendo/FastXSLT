# OASIS XSLT 1.0 Inherited `xml:space` Stripping

Date: 2026-09-23  
Status: Local compatibility and shared whitespace-view evidence

## Question

Can the accepted invocation-owned whitespace view apply exact
`xsl:strip-space elements="*"` semantics to sources containing inherited
`xml:space` declarations without mutating prepared XDM or adding a second
navigation path?

## Implemented boundary

The complete safe derived-document reference and the private visibility view
now carry one effective preserve/default bit while visiting source elements.
`xml:space="preserve"` protects whitespace-only text children of that element
and descendants; `xml:space="default"` restores the stylesheet's strip-all
policy for its subtree. The value is derived per invocation from immutable
source attributes and is not retained in prepared XDM or compiled state.

Both representations use the same inheritance rule. The visibility view still
retains only affected child-sequence overrides, preserves visible `NodeId`
values and provenance, and charges its existing one-pass XDM construction.
No parser stripping, source mutation, cross-invocation cache, or runtime
version branch was added.

CDATA lexical provenance remains intentionally outside this slice. FastXSLT's
XDM represents character data as text nodes, so this work does not introduce a
second text-node kind merely to reproduce one archival processor behavior.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases remain 2,260;
- successful executions rise from 2,196 to 2,208;
- execution failures fall from 64 to 52;
- exact XML comparisons rise from 2,046 to 2,054 / 3,173 (64.73%);
- eight unchanged cases become exact: Microsoft `Whitespaces__91430`,
  `91435`, `91436`, `91440`, `91447`, `91448`, `91451`, and `91452`;
- `Whitespaces__91429` and `91439` execute with the expected semantic counts
  but remain visible because the archival expected output inserts indentation
  between top-level result elements;
- `Whitespaces__91443` and `91444` execute and count 21 preceding text nodes
  rather than the archival 22 because the expected result preserves one
  whitespace-only CDATA-origin text node; `91444` is also marked doubtful by
  the suite; and
- comparison mismatches consequently rise from 67 to 71 while expected-error
  unexpected successes remain five and no execution panic occurs.

The movement from an explicit unsupported outcome to a visible mismatch is
intentional: the engine now executes the admitted source semantics and does
not conceal the remaining serializer or lexical-provenance differences.

## Verification

A focused runtime control compares the complete reference and visibility view
for inherited `preserve` and resetting `default` declarations. The local OASIS
sweep exercises child, self, following, preceding, preceding-sibling, and
following-sibling queries under the unchanged source files and stylesheets.
The archive remains local and unmodified; this is compatibility evidence, not
a conformance claim.
