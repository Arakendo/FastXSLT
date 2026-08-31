# XSLT30 `#all` and `#current` Node-Copy Case

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Selected case: `mode-1501`
- Principal stylesheet: `tests/attr/mode/mode-1501.xsl`
- Initial mode: `baz`
- Native assertion: XML-equivalent
  `<doc><para><?pi PI ?>text <bar><baz/></bar></para></doc>`

The unmodified stylesheet supplies a `match="node()" mode="#all"` rule whose
`xsl:copy` body recursively applies templates in `#current`. A more specific
`foo` rule in mode `baz` replaces that element with `bar`; the surrounding
document, elements, processing instruction, text, and nested `baz` element are
copied by the `#all` rule.

## Executable behavior

The existing expanded-mode and template-ranking path selects the `#all` rule
for each admitted node kind and preserves initial mode `baz` through every
`#current` application. The private `xsl:copy` executor now represents a copied
document by its constructed child sequence, retains the existing shallow
element construction, copies text through the shared bounded text path, and
constructs processing instructions through the existing result-node and byte
accounting path.

The exact native case passes XML comparison, including preservation of
`<?pi PI ?>` and selection of the mode-specific `foo` rule.

## Result

The complete mode denominator now records 38 passes, 44 profile exclusions,
and 87 visible default not-run cases out of 169. Across the 11 conserved
XSLT30 denominators, the total is 232 passes, 3 engine-unsupported cases, 49
profile exclusions, and 247 visible default not-run cases out of 531.

## Claim boundary

This slice covers `xsl:copy` for source document, element, text, and
processing-instruction contexts exercised by `mode-1501`. Attribute and comment
contexts remain unsupported by that private instruction path. The result-tree
representation still has no comment-node variant, and this evidence does not
admit general namespace-copy policy, copied source attributes, mode
`on-no-match` behavior, or other unselected `mode-15` cases.
