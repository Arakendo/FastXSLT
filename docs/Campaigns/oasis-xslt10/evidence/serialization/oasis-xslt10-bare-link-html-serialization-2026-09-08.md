# OASIS XSLT 1.0 Bare-Link HTML Serialization

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing bounded legacy HTML serializer admit the exact result shape
needed by the newly executable mixed-path AVT case without implying general
HTML serialization support?

## Changes

- The legacy HTML validator admits one unnamespaced `html` root containing one
  unnamespaced `a` element with exactly one unnamespaced `href` attribute and
  text-only content.
- Legacy HTML element-name recognition remains ASCII case-insensitive, so the
  source spelling `HTML` is preserved and accepted.
- The existing URI-attribute escaping, serialized-byte budget, cancellation,
  and result traversal remain the only execution path; no second serializer was
  introduced.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,475 | 1,475 | 0 |
| Executed successfully | 1,311 | 1,312 | +1 |
| Expected-result XML matches | 1,204 | 1,205 | +1 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 164 | 163 | -1 |
| `FXSR1001` execution frontier | 96 | 95 | -1 |

The unchanged `Lotus/attribvaltemplate_attribvaltemplate05#1` case now produces
the exact expected `<HTML><a href="..."></a></HTML>` result. No other case
changes disposition.

The strict expected-result lower bound is now 1,205 of 2,742 standard-operation
cases (43.95%) and 1,205 of all 3,173 catalog cases (37.98%). This is local
compatibility evidence, not a conformance claim.

## Boundaries

This tranche does not admit arbitrary HTML trees, arbitrary attributes,
comments or processing instructions in this shape, multiple links, implicit
HTML method selection, or a general HTML serializer guarantee. Every other
unrecognized HTML result continues to fail explicitly with `FXSR1001`.

## Verification

- A focused serializer test proves the bounded uppercase-root and URI-link
  result.
- The complete local OASIS measurement proves exactly one execution failure
  becomes one exact result with no mismatch or unrelated disposition change.
- No upstream corpus byte was edited.
