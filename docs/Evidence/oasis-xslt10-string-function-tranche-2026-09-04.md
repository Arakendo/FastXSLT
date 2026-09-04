# OASIS XSLT 1.0 String Function Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 647 definite unchanged XML passes; 913 initialized cases |
| Result | 653 definite unchanged XML passes; 920 initialized cases |
| Disposition | Shared typed string conversion; not a conformance claim |

## Change

The value compiler now folds `string()` over admitted static string, integer,
and boolean atoms and composes `string()` with the existing typed
location-path representation. Path evaluation is charged, requires zero or one
node, and obtains the complete charged XDM string value. An empty path produces
the empty string.

The path recognizer claims an expression only after successful typed-path
parsing. This preserves the existing scalar compiler for nested forms such as
`string(boolean(0))`; a before/after sweep caught and repaired an initial
dispatch-order regression before admission.

## Unchanged cases

Six Lotus cases move directly from initialization failure to XML comparison
pass:

- `string05` and `string37` exercise zero-or-one node paths;
- `string38` through `string41` exercise static integer and string atoms.

`string122` now initializes but reports `XPTY0004` because `av//*` supplies
multiple nodes. That is the modern function cardinality contract, not an engine
pass and not silent selection of the first node. Variable, decimal, nested
number conversion, and XSLT 1.0 compatibility coercions remain outside this
tranche.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 913 | 920 | +7 |
| Execution succeeded | 725 | 731 | +6 |
| XML comparison passes | 647 | 653 | +6 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 188 | 189 | +1 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The one additional execution failure is the intentional `string122`
cardinality boundary. The strict lower bound is now
**653 / 2,742 = 23.82%** of standard-operation cases and
**653 / 3,173 = 20.58%** of the complete catalog.

## Verification

A first-party production-path test covers static atoms, complete descendant
string value, an empty path, and conservation of nested scalar dispatch. A
separate test proves multi-node `XPTY0004`. The complete local OASIS sweep
confirms all six intended cases pass, the multi-node case remains visible, and
no mismatch, expected-error leak, or panic was introduced.
