# OASIS XSLT 1.0 static prefixed computed attribute -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the static computed-attribute namespace slice accept a prefixed QName while
resolving its expanded identity once during compilation and preserving the
existing result-namespace ownership model?

## Changes

- A static prefixed `xsl:attribute/@name` is resolved from its in-scope binding
  when no namespace override is present.
- A nonempty static `namespace` value overrides the lexical prefix binding for
  expanded-name identity.
- Empty namespaces on prefixed names, unbound prefixes, malformed QNames, and
  inconsistent use of the reserved `xml` prefix fail explicitly.
- Namespace AVTs remain unsupported. Runtime receives only an expanded name and
  the immutable owning-element namespace slice; it performs no QName parsing.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,551 | 1,562 | +11 |
| Executed successfully | 1,381 | 1,392 | +11 |
| Expected-result XML matches | 1,263 | 1,272 | +9 |
| XML comparison mismatches | 92 | 93 | +1 |
| XML comparator unsupported | 21 | 22 | +1 |
| Execution failures | 170 | 170 | 0 |

Microsoft `78372` is the newly visible mismatch; its expected attribute text
uses CRLF-derived tab/line-ending spelling that differs after ordinary XML
line-ending normalization. It remains uncredited. One other case reaches an
independent malformed-result/comparator boundary and also receives no credit.

The strict standard-operation lower bound is now
`1,272 / 2,742 = 46.39%`; the conservative all-catalog ratio is
`1,272 / 3,173 = 40.09%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not preserve a lexical prefix as result-node identity,
support computed name or namespace AVTs, add namespace aliasing, or generalize
nested dynamic attribute construction. Prefix choice remains serializer
presentation; expanded-name identity remains semantic. No corpus bytes or
expected results changed.

## Verification

- A focused compiler test proves static prefix resolution and namespace
  override semantics.
- The existing runtime namespace test continues to prove that a required
  compiled binding serializes safely.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
