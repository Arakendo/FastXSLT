# XSLT30 built-in-template typed empty parameter -- 2026-09-25

Date: 2026-09-25  
Status: Verified unchanged-corpus evidence  
Corpus: W3C XSLT 3.0 test suite revision `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`

## Question

Can the shared runtime distinguish a supplied zero-length string from a typed
empty sequence and carry both values through recursive built-in template rules?

## Implemented slice

- An empty `xsl:with-param` without `as` retains its zero-length string value.
- An empty `xsl:with-param as="xs:string?"` compiles to an empty atomic
  sequence. Other typed content remains outside this bounded slice.
- Invocation parameter transfer stores the empty sequence in the existing
  invocation-local atomic-sequence frame governed by ADR-0017; no new value
  store or cross-invocation sharing was introduced.
- `empty($variable)` observes sequence cardinality, while
  `$variable eq ''` requires one atomic value. Both paths remain work charged.
- Recursive built-in document and element rules carry the supplied parameter
  map unchanged until an explicit descendant template binds the parameters.

## Corpus result

The unchanged `misc/built-in-templates/built-in-templates-0201` case now
compiles, executes through the ordinary transform-set path, and matches its
native XML assertion. The complete six-case denominator moves from two to
three passes, with three cases still visibly not run.

The remaining cases require constructed sequence parameters or schema/type
annotation behavior. This tranche does not approximate either feature.

## Verification

- The focused built-in-template denominator tests execute all three selected
  unchanged cases.
- The overlay conserves all six case identities and records the new pass.
- The full workspace verification gate remains required before handoff.
