# OASIS XSLT 1.0 namespace-aware attribute match patterns -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the bounded template-pattern compiler distinguish attribute expanded names
from lexical prefixes for exact names, namespace wildcards, attribute-presence
predicates, and an element-with-attribute-value predicate?

## Implemented slice

The private match-pattern compiler now resolves `@prefix:name`, `@prefix:*`,
`*[@prefix:name]`, and `prefix:element[@name='value']` against the declaring
stylesheet element's namespace context. Exact and namespace-wildcard attribute
tests retain expanded-name semantics and the namespace wildcard keeps its XSLT
default priority. Source and temporary-tree matching use the same retained
namespace identities. An unbound prefix remains an `FXST0031` static error.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,663 | 1,666 | +3 |
| Executed successfully | 1,492 | 1,495 | +3 |
| Expected-result XML matches | 1,362 | 1,364 | +2 |
| XML comparison mismatches | 103 | 104 | +1 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 25 | 22 | -3 |

`namespace_namespace04` and `conflictres_conflictres19` now match their
unchanged archival expected XML. The latter exercises both `@ped:*` and the
higher-priority `@ped:x3` exact pattern. `Import__91048` advances through the
qualified element predicate but exposes a pre-existing result-namespace/XML
comparison mismatch and therefore remains visibly uncredited.

The strict standard-operation lower bound is now
`1,364 / 2,742 = 49.74%`; the conservative all-catalog ratio is
`1,364 / 3,173 = 42.99%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit qualified attribute value predicates, namespace-
wildcard element predicates, arbitrary QName-bearing XPath, general predicates,
or namespace serialization fixes. It does not change resource authority,
prepared-input ownership, invocation state, or public types.

## Verification

- Compiler coverage proves namespace resolution for exact and wildcard
  attributes and a qualified element predicate, plus static rejection of an
  unbound attribute prefix.
- Runtime coverage proves expanded-name dispatch for qualified elements and
  attributes.
- The complete 3,173-case local measurement produced the counters above.
