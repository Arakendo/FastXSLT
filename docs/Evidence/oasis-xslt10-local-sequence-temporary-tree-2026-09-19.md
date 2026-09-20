# OASIS XSLT 1.0 Local Sequence Temporary Trees

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

Local XSLT 1.0 variables without `select` construct a result tree fragment by
executing their sequence constructor. FastXSLT previously admitted only a few
hand-shaped forms: static text, one `xsl:value-of`, one text-only
`xsl:for-each`, or a fully static literal-element tree. A literal element that
contained an ordinary instruction therefore failed during initialization even
though the runtime already owned both complete instruction execution and
invocation-local temporary trees.

## Repair

- Keep the existing compact constructors as private optimized forms.
- When a local XSLT 1.0 content variable exceeds those forms, compile its body
  as the ordinary instruction sequence rather than introducing another
  evaluator.
- Execute that body into semantic result nodes, then materialize one
  invocation-owned temporary tree with fresh private identity.
- Preserve element names, namespace slices, attributes, child order, text,
  comments, and processing instructions during materialization.
- Charge every retained temporary element, attribute, text, comment, and
  processing-instruction node to the XDM-node budget before retention.
- Recurse through the constructor for namespace-alias application,
  decimal-format specialization, named-template validation, semantic
  inspection, and compiled-retention accounting.
- Keep modern content-variable behavior unchanged; the fallback is selected
  only by XSLT 1.0 compatibility static context.

Focused tests cover dynamic literal content and attributes, result-tree-fragment
string conversion, `xsl:copy-of`, XDM-budget exhaustion, and the modern-version
non-admission boundary.

## Corpus result

Against the unchanged 3,173-case catalog:

- initialized: 2,033 -> 2,042;
- successfully executed: 1,934 -> 1,940;
- exact XML-semantic matches: 1,805 -> 1,811;
- visible XML mismatches: unchanged at 54;
- `FXST1015` initialization frontier: 30 -> 21;
- execution failures: 99 -> 102.

The last increase is expected and useful: three cases formerly stopped at the
over-broad constructor boundary and now reach distinct, visible runtime
frontiers. Six newly executable unchanged cases compare exactly. The Lotus
`variable03` and `variable04` cases are representative dynamic local
constructor examples.

## Boundary conclusion

This tranche establishes ordinary local XSLT 1.0 result-tree-fragment
construction through the existing instruction semantics and invocation-owned
temporary-tree representation. It does not widen global constructors, define
general modern sequence construction, make temporary trees shareable across
invocations, or expose a public tree/provider abstraction.
