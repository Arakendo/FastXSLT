# OASIS XSLT 1.0 Variable String and Number Conversion Tranche

Date: 2026-09-06  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the shared compiler and runtime perform the XPath 1.0 `string()` and
`number()` conversions of variables without weakening modern XPath
cardinality/type behavior or creating a second compatibility engine?

## Change

The value compiler now recognizes `string($name)` and `number($name)` only
when the stylesheet selects the XSLT 1.0 compatibility mode. The runtime uses
the existing invocation/global variable owners and applies the XPath 1.0
conversion rules to atomic singleton values, source node sets, temporary
result trees, and empty sequences/result trees. Source node sets use their
first node in document order.

The number conversion admits the existing bounded XPath decimal lexical
grammar. Invalid or empty strings produce `NaN`, and positive or negative zero
serializes as `0`. Modern stylesheets continue through the modern typed/path
compiler and do not inherit this compatibility conversion.

The focused executable sentinel covers atomic, empty temporary-tree,
source-derived temporary-tree, and multi-node source-variable conversions.
All source and temporary-tree string-value traversal continues to use the
existing work-control charge points.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,281 | 1,287 | +6 |
| Executed successfully | 1,108 | 1,114 | +6 |
| Expected-result XML matches | 996 | 1,002 | +6 |
| XML comparison mismatches | 77 | 77 | 0 |
| Execution failures | 173 | 173 | 0 |

The targeted `Lotus/string_string43#1` and doubt-annotated
`Lotus/math_math18#1` cases now execute and agree with their unchanged
archival expected results. The corpus measurement also discovers four sibling
uses through the same typed conversion boundary. No expected result or
upstream corpus byte was edited.

The strict standard-operation lower bound is now
`1,002 / 2,742 = 36.54%`; the deliberately conservative all-catalog ratio is
`1,002 / 3,173 = 31.58%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Limits retained

- This tranche does not implement the general XPath 1.0 conversion lattice.
- The numeric lexical scope remains bounded and does not claim exponent or
  non-decimal lexical conversion.
- `math18` retains its suite doubt metadata even though output agrees.
- Complex temporary-tree constructors remain outside the admitted global
  temporary-value slice.
- No public type, host policy, resource authority, or execution topology
  changed.

## Verification

- Focused runtime conversion sentinel passed.
- The complete local OASIS measurement completed with the counters above.
- Full workspace verification is recorded after the surrounding campaign
  tranche is closed.
