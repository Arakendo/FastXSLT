# XSLT30 parentless attribute mode policies -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `mode-0007` case. Its source-free
invocation constructs one typed parentless attribute, applies the declared
`shallow-copy`, `shallow-skip`, and `text-only-copy` policies, and matches the
native XML assertion.

## Implemented semantics

- One static unprefixed `xsl:attribute` constructor is admitted for an exact
  global `as="attribute()"` declaration and materialized as immutable
  invocation-owned temporary state.
- Shallow-copy produces a private pending result attribute. The immediately
  containing literal result element consumes that item into its attribute set;
  it is never represented as a child node.
- Attribute construction is rejected as `XTDE0410` when it follows a child,
  duplicates an existing expanded attribute name, or escapes a containing
  element into serialization.
- Shallow-skip omits the parentless attribute. Text-only-copy emits its string
  value as result text.
- Temporary XDM construction and resulting attribute/text values retain their
  existing work-domain accounting.
- Known compiled-state retention now includes both constructed-element literal
  attributes and parentless temporary-attribute names and values.

## Boundary

The parentless constructor remains static: prefixed or computed names, dynamic
content, namespace nodes, and general attribute sequences are unsupported.
The pending result item is private construction state, not a semantic node
kind, public result API, or serializer extension.

## Denominator movement

The complete 169-case `attr/mode` denominator moves from 80 to 81 passes and
from 44 to 43 visible default not-run cases; its 45 profile exclusions are
unchanged. Across the eleven conserved XSLT30 denominators, the total moves
from 400 to 401 passes and from 77 to 76 visible default not-run cases, with
three engine-unsupported cases and 51 profile exclusions unchanged.
