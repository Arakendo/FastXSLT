# OASIS XSLT 1.0 Parenthesized-Path and Escaped-Brace AVTs -- 2026-09-15

Date: 2026-09-15  
Status: Verified shared semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a literal-result attribute value template combine one parenthesized
location path with an escaped literal brace without introducing a general
XPath expression evaluator?

## Implemented slice

The private typed AVT scanner now admits one dynamic path when the same AVT
also contains an escaped `{{` or `}}` brace. One nonempty outer pair of
parentheses may surround that path. The inner expression must still compile as
an existing typed location path; nested expression grammar, arithmetic, and
general function calls remain outside this slice.

This preserves the earlier specialized representation for ordinary one-path
AVTs. Malformed single braces and empty expressions remain rejected, and path
execution retains the existing traversal charges and first-node string
conversion.

## Corpus result

Three unchanged Microsoft cases now initialize and execute:

- `AVTs__77577` combines an escaped opening brace with a later dynamic path;
- `AVTs__77579` emits an escaped opening brace before parenthesized `@name`;
- `AVTs__77580` emits parenthesized `@name` before an escaped closing brace.

All three produce the expected attribute values. They remain visible,
uncredited comparison mismatches because their expected fragments contain
serializer-added whitespace between top-level elements while FastXSLT emits
the same elements contiguously. The XML comparator was not weakened.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,767 | 1,770 | +3 |
| Initialization failures | 1,368 | 1,365 | -3 |
| Executed successfully | 1,583 | 1,586 | +3 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,444 | 1,444 | 0 |
| XML comparison mismatches | 110 | 113 | +3 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound remains
`1,444 / 2,742 = 52.66%` for standard-operation cases and
`1,444 / 3,173 = 45.51%` for the complete catalog.

## Verification

Focused compiler tests preserve the typed AVT shape and the pre-existing
specialized one-path representation. A production runtime test executes both
escaped-brace directions through compile, transform, and serialization. The
complete local corpus sweep establishes the conserved counts and absence of a
new execution failure or panic.
