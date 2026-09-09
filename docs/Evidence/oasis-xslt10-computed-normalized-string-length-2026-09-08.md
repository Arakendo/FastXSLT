# OASIS XSLT 1.0 computed normalized-string length -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a computed attribute evaluate the exact XSLT 1.0
`string-length(normalize-space(.))` composition while preserving source string
value semantics, XML whitespace rules, Unicode codepoint length, and bounded work
accounting?

## Changes

- The XSLT 1.0 computed-attribute compiler admits only the exact context-item
  composition; it does not infer a general nested-function evaluator.
- Runtime visits the source node string value through controlled XDM traversal,
  collapses only XML whitespace characters, and counts normalized Unicode scalar
  values without constructing an intermediate normalized string.
- Every visited character remains charged as XPath work, and integer overflow is
  reported as structured `FOAR0002`.
- Missing or non-source focus is a structured `XPDY0002` failure.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,501 | 1,502 | +1 |
| Executed successfully | 1,336 | 1,337 | +1 |
| Expected-result XML matches | 1,228 | 1,229 | +1 |
| XML comparison mismatches | 82 | 82 | 0 |
| Execution failures | 165 | 165 | 0 |

The unchanged Lotus `string140` case now reports the expected normalized length
for all twelve source elements and preserves the already-supported normalized
text value. Its archival CRLF versus generated LF spelling is correctly handled
by the XML infoset comparator.

The strict standard-operation lower bound becomes
`1,229 / 2,742 = 44.82%`; the conservative all-catalog ratio becomes
`1,229 / 3,173 = 38.73%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit arbitrary nested string functions, path arguments,
temporary-tree focus, or general computed-attribute expressions. It does not
change `normalize-space()` or `string-length()` semantics elsewhere. No corpus
bytes or expected results were changed.

## Verification

- The focused runtime test covers whitespace spanning descendant text nodes and
  a multi-byte Unicode scalar, proving codepoint rather than byte length.
- The unchanged `string140` case passes its expected XML comparison.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
