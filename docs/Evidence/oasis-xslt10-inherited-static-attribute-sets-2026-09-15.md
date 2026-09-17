# OASIS XSLT 1.0 Inherited Static Attribute Sets -- 2026-09-15

Date: 2026-09-15  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the compile-time local static attribute-set representation compose
`use-attribute-sets` references without introducing runtime lookup state or
silently accepting undefined and circular graphs?

## Implemented slice

Yes. The compiler now validates the complete local attribute-set dependency
graph and expands referenced sets recursively. Referenced sets are applied in
lexical list order, then the referring set's own attributes override them.
Attributes on the consuming literal or computed element retain their existing
higher precedence.

Undefined references fail with `XTSE0710`; direct and indirect cycles fail with
`XTSE0720`. Expansion remains limited to one stylesheet document, uniquely
named declarations, and static-text attribute values. It does not add a runtime
registry, cross-module declaration merging, or dynamic set values.

## Corpus result

Seven unchanged cases become exact XML-semantic expected-result passes: Lotus
`attribset05`, `attribset06`, `attribset07`, `attribset08`, `attribset33`, and
`attribset34`, plus Microsoft `AttributeSets_EmptyAttribSet`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,794 | 1,803 | +9 |
| Initialization failures | 1,341 | 1,332 | -9 |
| Executed successfully | 1,606 | 1,613 | +7 |
| Execution failures | 188 | 190 | +2 |
| Expected-result XML matches | 1,461 | 1,468 | +7 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The two additional execution failures are Microsoft cases that now reach the
already-known unsupported UTF-16 string-serialization boundary. The expected
error identities for undefined and circular graphs remain observed failures,
now under the graph-specific diagnostics instead of the former generic
unsupported-attribute boundary.

The exact compatibility lower bound rises to
`1,468 / 2,742 = 53.54%` for standard-operation cases and
`1,468 / 3,173 = 46.27%` for the complete catalog.

## Verification

A focused production-lifecycle test verifies three-level inheritance, local-set
override, and final explicit-attribute override. Compiler tests cover undefined,
self-circular, and indirectly circular graphs. The unchanged complete sweep
conserves all 3,173 identities and adds no XML mismatch or panic.

The graph owner was extracted from the source unit that performs general
instruction traversal; its cohesion and remaining size pressure are recorded
separately in the
[attribute-set compiler decomposition evidence](attribute-set-compiler-decomposition-2026-09-16.md).
