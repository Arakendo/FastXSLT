# OASIS XSLT 1.0 Lexical Node-Name Prefix Retention

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `name()` return the source lexical QName for namespaced elements and
attributes without reconstructing a prefix from namespace bindings?

## Change

The XML adapter now retains an attribute's lexical prefix separately from its
expanded name, just as it already did for elements. Prepared XDM construction
moves that prefix into the immutable source node and includes its capacity in
retained-memory accounting.

The existing context-node and typed-path `name()` operations now render the
retained `prefix:local` spelling. Expanded-name identity remains namespace URI
plus local name; the prefix is lexical provenance only. The runtime does not
invent a prefix, search for an equivalent binding, or change QName comparison
semantics.

This supersedes the implementation boundary recorded in the 2026-09-04 name-
path evidence. That earlier record remains an accurate historical observation
of the then-current representation.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,376 | 1,376 | 0 |
| Executed successfully | 1,208 | 1,215 | +7 |
| Expected-result XML matches | 1,094 | 1,101 | +7 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 168 | 161 | -7 |
| `FXRT1008` observations | 7 | 0 | -7 |

All seven former lexical-prefix runtime failures now match their unchanged
expected results. `Lotus/string_string33#1` and `string34#1` are representative
element-name cases; focused coverage also proves namespaced attribute names.

The strict standard-operation lower bound is now
`1,101 / 2,742 = 40.15%`; the deliberately conservative all-catalog ratio is
`1,101 / 3,173 = 34.70%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused XML/XDM coverage proves equivalent expanded names can retain distinct
  element and attribute lexical prefixes.
- Focused runtime coverage proves context `name()` and path `name()` return the
  retained element and attribute spelling.
- The complete local OASIS measurement completed with the counters above and no
  upstream corpus byte was edited.

