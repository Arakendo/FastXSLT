# OASIS XSLT 1.0 explicit-axis boolean paths -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can all explicit-axis location paths already admitted by the typed XPath parser
participate in instruction boolean expressions without adding axis-specific
boolean plans?

## Implemented slice

Instruction boolean compilation now attempts the existing typed location-path
parser for every expression containing an explicit `::` axis separator. A
success lowers to the existing node-set effective-boolean-value plan; malformed
or unsupported paths still fall through to the existing structured unsupported
diagnostic.

No new axis evaluator or XSLT 1.0-only runtime behavior was introduced. The
same controlled path evaluator determines whether the selected node sequence is
empty in every stylesheet version.

## Corpus result

Unchanged Lotus `axes_axes130` moves from initialization failure to successful
execution. In an attribute context, the four predicates produce the required
decisions:

- `self::node()` is true;
- `self::*` is false because the self axis has element as its principal node
  type for the wildcard test;
- `self::text()` is false;
- `self::center-attr` is false for the same principal-node-type reason.

The semantic result is correct. The case remains an XML comparison mismatch
because the stylesheet requests `indent='yes'`, FastXSLT emits two-space
indentation, and the archival expected artifact uses line breaks without those
spaces. XSLT 1.0 does not standardize the indentation amount. The current local
comparator conservatively retains whitespace text nodes, so this tranche does
not credit the case as an exact pass.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,728 | 1,729 | +1 |
| Initialization failures | 1,407 | 1,406 | -1 |
| Executed successfully | 1,545 | 1,546 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,411 | 1,411 | 0 |
| XML comparison mismatches | 107 | 108 | +1 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound remains
`1,411 / 2,742 = 51.46%` for standard-operation cases and
`1,411 / 3,173 = 44.47%` for the complete catalog. This is executable breadth
and comparison-harness evidence, not an inflated conformance credit.

## Verification

A focused end-to-end test executes all four self-axis predicates against an
attribute and checks their exact text result. Existing XPath path tests continue
to cover the typed self-axis node tests. The complete local corpus sweep confirms
the one-case frontier movement, unchanged execution-failure count, and absence
of panics.
