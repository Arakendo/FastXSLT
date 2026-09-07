# OASIS XSLT 1.0 Typed Context and Path Sort Keys

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:sort` reuse already admitted lexical-name, context string-value, and
controlled location-path semantics without widening sort keys to a general
dynamic expression evaluator?

## Change

The private sort-key plan now admits three additional typed forms:

- `name()` and `name(.)` charge a node visit and render the retained source
  lexical prefix and local name;
- `string-length()` and `string-length(.)` traverse the controlled context
  string value and charge every Unicode codepoint;
- `count(path)` evaluates an owned typed location path and charges the count
  operation after controlled navigation.
- `number(path)` evaluates the same controlled path, selects XSLT 1.0 first-node
  conversion at compilation, and reuses the existing canonical finite/`NaN`
  lexical conversion.

Sort stability, text/numeric conversion, focus, and selected-node ownership
remain unchanged.

This does not admit arbitrary function-valued sort keys, completion-order sort,
or a second sorting evaluator.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,376 | 1,380 | +4 |
| Executed successfully | 1,215 | 1,219 | +4 |
| Expected-result XML matches | 1,101 | 1,105 | +4 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/select_select69#1`, which sorts attributes
by lexical name including empty values; `Lotus/sort_sort24#1`, which sorts by
context string length; and `Lotus/sort_sort25#1`, which counts following
siblings; and `Microsoft/Sorting__83821#1`, which sorts canonical numbers and a
`NaN` conversion. The generic function-shaped initialization frontier falls
from 97 to 94 observations. One additional parser-invalid observation becomes
executable through the newly recognized `count(path)` boundary, so no case
disappears from the accounting.

The strict standard-operation lower bound is now
`1,105 / 2,742 = 40.30%`; the deliberately conservative all-catalog ratio is
`1,105 / 3,173 = 34.83%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused runtime coverage sorts differently named, namespaced source elements
  and attributes by `name(.)` and verifies retained lexical prefixes in both
  resulting orders.
- The same focused lifecycle sorts elements by controlled Unicode string length
  and controlled child counts, then sorts finite and nonnumeric attribute
  values through the XSLT 1.0 `number(path)` conversion.
- The existing multi-key, numeric, positional, stable-order, and apply-template
  sort coverage continues through the same owner.
- The complete local OASIS measurement completed with the counters above and no
  upstream corpus byte was edited.
