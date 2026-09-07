# OASIS XSLT 1.0 Path String-Function Tranche

Date: 2026-09-06  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can common XPath 1.0 binary string functions over source paths reuse the
shared location-path and controlled string-value machinery without admitting a
general dynamic function evaluator?

## Change

The XSLT 1.0 value compiler now recognizes four bounded path-plus-literal
forms:

- `contains(path, 'literal')`;
- `starts-with(path, 'literal')`;
- `substring-before(path, 'literal')`; and
- `substring-after(path, 'literal')`.

The path is compiled with the existing namespace-aware location-path owner.
At execution, XPath 1.0 conversion selects the first node in document order,
uses its controlled string value, or uses the empty string for an empty node
set. The full path and string traversal remain charged, followed by one XPath
operation charge for the selected string operation. Modern stylesheet
behavior is unchanged.

This is deliberately not a general function-call AST. Dynamic second
operands, nested calls, broader arities, and the rest of the XPath 1.0 string
function surface remain explicit later work.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,287 | 1,306 | +19 |
| Executed successfully | 1,114 | 1,133 | +19 |
| Expected-result XML matches | 1,002 | 1,021 | +19 |
| XML comparison mismatches | 77 | 77 | 0 |
| Execution failures | 173 | 173 | 0 |

The tranche removes nine cases from the measured
`location-path-shape:function` initialization frontier; other cases were
previously grouped under the broader `FXXP1001` frontier. Every newly executed
case agrees with its unchanged archival expected result. No upstream corpus
byte was edited.

The strict standard-operation lower bound is now
`1,021 / 2,742 = 37.24%`; the deliberately conservative all-catalog ratio is
`1,021 / 3,173 = 32.18%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Conservation and limits

- The plan is selected only for XSLT 1.0 compatibility mode.
- Node-set conversion uses the first node in document order as required by
  XPath 1.0 rather than weakening modern sequence cardinality.
- Empty-string behavior for all four functions is covered by the focused
  runtime sentinel.
- Source traversal, result construction, diagnostics, cancellation, and work
  budgets remain on their existing owners.
- No public API, resource authority, or host scheduling behavior changed.

## Verification

- Focused path string-function runtime sentinel passed.
- Strict crate Clippy passed after the implementation.
- The complete local OASIS measurement completed with the counters above.
- Full workspace verification is recorded after the surrounding campaign
  tranche is closed.
