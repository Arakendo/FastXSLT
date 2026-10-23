# XSLT30 call-template modern string predicate

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 case `call-template-1901` passes through the normal
sealed-resource, compilation, transform-set, and XML-comparison path.

Four calls supply different constructed values to the same named-template
parameter. The called template evaluates
`./action[@priority=string($x)]`, proving both source-child attribute lookup and
modern `string()` conversion of the invocation-owned temporary document. The
four results also prove that one invocation frame does not retain a previous
call's parameter value.

FastXSLT now gives the modern predicate its own typed operation. Its `string()`
conversion accepts zero or one atomic or source item, accepts one temporary
document, reports `XPTY0004` for multi-item sequences, and remains separate from
the XSLT 1.0 first-item compatibility operation.

## Boundary

The admitted predicate remains one unnamespaced child name, one unnamespaced
attribute name, and one unqualified variable inside an explicit `string()`
call. General predicates, namespace-sensitive names, implicit general
comparisons, and expanded-QName variable identity remain outside this tranche.

## Accounting

The complete 42-case `insn/call-template` denominator advances from 27 to 28
passes, with one profile exclusion and 13 visible defaults. Across the conserved
XSLT30 denominators this produces 499 passes, 12 engine-unsupported cases,
2,879 profile exclusions, and 109 visible defaults out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt30_call_template_inventory_tests --all-features`
