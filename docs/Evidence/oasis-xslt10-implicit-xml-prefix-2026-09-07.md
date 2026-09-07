# OASIS XSLT 1.0 Implicit `xml` Prefix

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Do instruction XPath expressions recognize the reserved `xml` prefix without
requiring an explicit namespace declaration?

## Change

The instruction-expression and apply-selection namespace resolvers now return
the namespace name fixed by Namespaces in XML for the `xml` prefix:
`http://www.w3.org/XML/1998/namespace`. Ordinary lexical ancestor lookup remains
unchanged for every other prefix.

This repairs static QName resolution rather than adding an XSLT 1.0-only
compatibility branch. The parsed source tree already retains XML-namespaced
attributes by expanded name.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,406 | 1,407 | +1 |
| Executed successfully | 1,245 | 1,246 | +1 |
| Expected-result XML matches | 1,130 | 1,131 | +1 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact case is `Lotus/attribset_attribset20#1`, which selects both
an explicitly declared qualified attribute and `@xml:att1` without declaring
the reserved prefix.

The strict standard-operation lower bound is now
`1,131 / 2,742 = 41.25%`; the deliberately conservative all-catalog ratio is
`1,131 / 3,173 = 35.64%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- The focused qualified value-path lifecycle test now also selects
  `doc/@xml:lang` without an explicit `xmlns:xml` declaration.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
