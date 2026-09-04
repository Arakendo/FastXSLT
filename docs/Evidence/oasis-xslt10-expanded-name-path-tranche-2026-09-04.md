# OASIS XSLT 1.0 Expanded-Name Path Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 609 definite unchanged XML passes; 874 initialized cases |
| Result | 613 definite unchanged XML passes; 879 initialized cases |
| Disposition | Shared path-dependent XPath semantics; not a conformance claim |

## Change

`local-name()` and `namespace-uri()` now accept admitted principal-source
location paths. Qualified child steps resolve their prefixes in stylesheet
static context; unqualified steps continue to honor the effective XPath default
namespace. Runtime evaluation selects a zero-or-one node and reads its retained
expanded name, avoiding lexical-prefix reconstruction.

The path evaluator and final node inspection remain work charged. Empty
selection produces an empty string for either function.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 874 | 879 | +5 |
| Execution succeeded | 687 | 691 | +4 |
| XML comparison passes | 609 | 613 | +4 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 188 | +1 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **613 / 2,742 = 22.36%** of
standard-operation cases and **613 / 3,173 = 19.32%** of the complete catalog.
Unchanged OASIS `Lotus/node_node04#1`, `Lotus/namespace_namespace07#1`,
`Lotus/namespace_namespace10#1`, and `Lotus/namespace_namespace12#1` reach
definite XML comparison passes. `Lotus/node_node05#1` initializes but remains a
visible `XPTY0004` execution failure because its wildcard selects multiple
nodes.

## Boundaries

The shared modern zero-or-one cardinality is intentional. XPath 1.0's
first-node conversion for larger node-sets remains behind AR-0019's
version-aware compatibility review. Qualified attribute steps, namespace nodes,
variables, temporary trees, and general expressions remain explicit. Unlike
`name()`, these functions return expanded-name components and therefore do not
require preservation or reconstruction of a source lexical prefix.
