# XSLT30 overlapping union single-rule mode semantics -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `mode-1516` and `mode-1517`
cases over the native `mode-15` source. Both stylesheets declare named mode
`c` with `on-multiple-match="fail"`; their `para[foo] | para[text()]` pattern,
with and without enclosing parentheses, matches both alternatives for one
source `para`. The result is nevertheless the expected
`<out><unambiguous-match/></out>` because the alternatives belong to one
template rule rather than two competing rules.

## Implemented semantics

- The compiled mode policy retains `on-multiple-match="fail"` as stylesheet
  semantics. It overrides a host-supplied recover/use-last policy for source
  and temporary-tree selection, including `xsl:next-match`.
- Equal-default-priority union alternatives that may overlap remain one
  compiled template identity. A node matching several branches therefore
  contributes one candidate, one selection rank, and one invocation.
- Exact child-presence predicates `QName[QName]` and `QName[text()]` inspect
  typed child nodes and charge each inspected node to the XPath-node-visit
  work domain.
- A catalog invocation that supplies both a named initial template and a
  principal source now prepares that source and provides its document node as
  the initial context. Source-free initial-template behavior remains intact.
- A focused negative control compiles two distinct equal-rank rules matching
  the same node and proves that the mode-owned policy reports `XTDE0540` even
  when the private host fallback requests recovery.

## Deliberate boundary

This slice does not admit general predicate grammar or arbitrary union-pattern
grammar. Overlapping alternatives with different default priorities remain
explicitly unsupported, while already admitted provably disjoint unions keep
their existing representation. The new child tests are exact presence tests;
they do not establish general effective-boolean-value predicate evaluation.

## Conservation

The complete 169-case `attr/mode` denominator moves from 83 to 85 passes and
from 41 to 39 visible default not-run cases; its 45 profile exclusions remain
unchanged. Across the eleven conserved XSLT30 denominators, the total moves
from 403 to 405 passes and from 74 to 72 visible default not-run cases, with
three engine-unsupported cases and 51 profile exclusions unchanged.
