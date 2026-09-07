# OASIS XSLT 1.0 Qualified Value Paths

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can ordinary value-expression compilation reuse the namespace-aware qualified
child/attribute path owner already used by copy, node-name, and local-variable
operations?

## Change

The final value-expression location-path fallback now resolves simple explicit
QName paths against the stylesheet element's in-scope namespaces. The admitted
shape contains only child or attribute QName steps separated by `/`; it excludes
wildcards, explicit axes, predicates, and function syntax.

XSLT 1.0 first-node string conversion remains selected by the existing static
compatibility mode. Path evaluation, namespace identity, work charging, and
unbound-prefix diagnostics remain owned by the existing typed path machinery.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,398 | 1,400 | +2 |
| Executed successfully | 1,237 | 1,239 | +2 |
| Expected-result XML matches | 1,123 | 1,125 | +2 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Microsoft/Namespace__77655#1` and
`Microsoft/ConflictResolution__77619#1`. Both exercise explicit namespace
prefixes in ordinary value selection while retaining their surrounding
namespace/result semantics.

The strict standard-operation lower bound is now
`1,125 / 2,742 = 41.03%`; the deliberately conservative all-catalog ratio is
`1,125 / 3,173 = 35.46%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused lifecycle test selects both `doc/p:item` and
  `doc/p:item/@p:code` through `xsl:value-of` and verifies their string values.
- The complete local OASIS measurement completed with the counters above and no
  upstream corpus byte was edited.
