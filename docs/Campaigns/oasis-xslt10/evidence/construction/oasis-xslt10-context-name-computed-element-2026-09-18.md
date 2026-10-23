# OASIS XSLT 1.0 Context-Name Computed Element

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the shared result-construction path admit the exact dynamic
`xsl:element name="{name()}"` / `name="{name(.)}"` forms without treating the
returned lexical QName as an already-authorized expanded name?

## Result

Yes, for a deliberately bounded first slice.

- Compilation retains the optional static `namespace` override and the
  `xsl:element` instruction's in-scope namespace bindings.
- Execution obtains the context node's lexical prefix and local name, resolves
  that prefix against the retained stylesheet static context, and constructs
  the ordinary semantic result element.
- An explicit static namespace override replaces namespace lookup. An
  unprefixed dynamic name otherwise uses the retained default binding when one
  exists.
- A prefixed value whose prefix is not bound by the stylesheet static context
  fails as `XTDE0830 / invalid`; an empty node name fails as
  `XTDE0820 / invalid`.
- Result-node and XPath-node work remain charged, and the existing element body,
  computed-attribute, namespace-fixup, serialization, budget, and cancellation
  paths remain authoritative.

This does not admit arbitrary name AVTs, path-valued names, variable-valued
names, dynamic namespace AVTs, namespace-axis nodes, or a public dynamic-QName
representation.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- The generic `FXST1047` dynamic-name frontier falls from 28 to 21 cases.
- One unchanged expected-error case,
  `Microsoft/Elements_Element_ElementWithNSFromNameFn#1`, now initializes and
  reports the required runtime `XTDE0830` because its source lexical prefix is
  not bound to that prefix in the stylesheet static context.
- Three newly reached cases expose independently invalid principal-source XML,
  while the remaining advanced cases expose later initialization boundaries.
- Initialized cases rise from 1,939 to 1,940; execution failures rise from 95
  to 96; expected initialization-error observations move from 401 to 400 and
  expected execution-error observations move from 22 to 23.
- Exact comparisons remain 1,565 and mismatches remain 208. The slice improves
  semantic classification and unlocks later work; it does not manufacture an
  exact-result gain.

## Verification

- Focused execution constructs a prefixed result element only after resolving
  the source lexical prefix through the stylesheet's static namespace context.
- The full local OASIS measurement conserves all catalog identities and exposes
  the later dispositions above.
- The ordinary workspace verification gates pass.
