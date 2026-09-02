# XSLT30 parentless comment, PI, and text mode policies -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `mode-0001`, `mode-0003`, and
`mode-0005` cases. Each source-free invocation enters named template `main`,
materializes one typed parentless temporary node, and applies the declared
`shallow-copy`, `shallow-skip`, and `text-only-copy` built-in mode policies.
The native XML assertions pass for comment, processing-instruction, and text
nodes.

## Implemented semantics

- A global variable may use one static `xsl:comment`,
  `xsl:processing-instruction`, or `xsl:text` constructor when its exact
  sequence type is respectively `comment()`, `processing-instruction()`, or
  `text()`.
- The resulting node is immutable, invocation-owned, parentless, and charged
  to the XDM work domain during materialization.
- Temporary-tree template selection recognizes comment,
  processing-instruction, text, and `node()` patterns by node kind.
- `shallow-copy` copies the selected leaf node, `shallow-skip` contributes
  nothing, and `text-only-copy` copies only the text node's string value.
- Copied result nodes retain result-node and text-byte accounting.

The corpus adapter also now honors an explicit catalog `initial-template`
without manufacturing a principal source. Existing source and initial-mode
entries continue through their previous paths.

## Boundary

This slice does not admit typed global expressions generally. The constructor
must be one statically known node and its declared type must match exactly.
Parentless attributes remain outside the slice because copying an attribute
into a containing result element requires an owned result-tree attribute
attachment path, not a child-node approximation. Namespace nodes remain
outside the current XDM representation.

## Denominator movement

The complete 169-case `attr/mode` denominator moves from 76 to 79 passes and
from 48 to 45 visible default not-run cases; its 45 profile exclusions are
unchanged. Across the eleven conserved XSLT30 denominators, the total moves
from 396 to 399 passes and from 81 to 78 visible default not-run cases, with
three engine-unsupported cases and 51 profile exclusions unchanged.
