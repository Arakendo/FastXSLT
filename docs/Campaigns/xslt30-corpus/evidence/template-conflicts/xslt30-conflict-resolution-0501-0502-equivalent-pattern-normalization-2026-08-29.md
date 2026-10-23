# XSLT30 `conflict-resolution-0501`–`0502` Equivalent Pattern Normalization

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Cases: `conflict-resolution-0501`, `conflict-resolution-0502`
- Stylesheets: `conflict-resolution-0501.xsl`, `conflict-resolution-0502.xsl`
- Source: inline `conflict-resolution-05` environment

## Representation and execution

The two standard cases express one match property through different XPath
surface forms:

- `0501`: `*[*[name()=name(current())]]`
- `0502`: `*[some $x in child::* satisfies name($x) = name(.)]`

Compilation recognizes those exact forms and lowers both to one typed
`ElementWithSameNamedChild` pattern operation. No lexical pattern parsing or
general `current()`/quantified-expression evaluation occurs in the dispatch
loop.

Runtime selection charges each inspected child and compares lexical element
names only in the admitted unnamespaced domain. The first outer `a` has a child
also named `a`, selects the predicate rule, and uses the previously admitted
source-copy construction to add `recursive="yes"`. The second outer `a` has
only a `b` child and selects the wildcard rule.

## Result

| Case | Expected distinguishing result | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0501` | first outer `a` has `recursive="yes"` | semantically equal | passed |
| `conflict-resolution-0502` | first outer `a` has `recursive="yes"` | semantically equal | passed |

## Claim boundary

This evidence admits only the two exact standard pattern forms above and their
shared unnamespaced same-named-child semantics. It does not admit general
`current()`, quantified expressions, arbitrary effective boolean values,
namespace-sensitive `name()` comparison, or the parent/positional variants in
`0503` and `1501`. Encountering namespaced candidates in this private operation
fails explicitly instead of approximating lexical QName behavior with expanded
names.
