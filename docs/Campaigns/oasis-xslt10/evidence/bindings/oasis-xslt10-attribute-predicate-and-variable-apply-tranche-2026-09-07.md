# OASIS XSLT 1.0 Attribute Predicate and Variable Apply Tranche

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the shared location-path and template-application machinery admit the
standard abbreviated attribute predicates `[@name]` and
`[@name='literal']`, then apply templates directly to an existing source-node
variable without adding a legacy path evaluator or temporary-tree conversion?

## Change

The private typed final-axis predicate now recognizes `@name` as the
abbreviation of `attribute::name`. It can also retain one static string value
for exact attribute equality. Evaluation continues to scan the candidate
element's real attribute axis and charges every inspected attribute as an XPath
node visit. Presence and equality therefore use one bounded typed operation;
attributes do not become children and no source tree is copied.

Literal equality is deliberately narrow. This tranche does not admit dynamic
predicate operands, numeric/general comparisons, multiple predicates, boolean
predicate composition, namespace-qualified attribute names, or a general
predicate AST.

The adjacent `xsl:apply-templates select="$variable"` path now resolves the
variable's existing runtime value kind. A source-node sequence is applied in
document order with its ordinary focus position and size; a temporary-tree
document continues through the existing temporary-tree executor. No conversion
or sibling visibility is introduced.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,333 | 1,340 | +7 |
| Executed successfully | 1,172 | 1,179 | +7 |
| Expected-result XML matches | 1,061 | 1,067 | +6 |
| XML comparison mismatches | 76 | 77 | +1 |
| Execution failures | 161 | 161 | 0 |

Representative unchanged passes include:

- `Lotus/axes_axes64#1`, exercising `self::*[@center-attr-2]`;
- `Lotus/select_select10#1`, selecting a node-set through literal attribute
  equality;
- `Lotus/select_select81#1`, applying templates to relative and absolute
  literal-equality selections; and
- `Lotus/select_select08#1`, applying templates directly to a global
  source-node variable selected by `/doc/*[@test='true']`.

The dominant generic `FXXP1001` initialization frontier falls from 155 to 137
observations. Cases that merely advance to a different unsupported or invalid
boundary remain uncredited.

`Microsoft/Sorting__77526#1` is the one newly exposed comparison mismatch. Its
sorting semantics execute, but its result differs in HTML empty-element and
whitespace serialization. That obligation remains visible and receives no pass
credit.

The strict standard-operation lower bound is now
`1,067 / 2,742 = 38.91%`; the deliberately conservative all-catalog ratio is
`1,067 / 3,173 = 33.63%`. These are compatibility measurements, not an XSLT 1.0
conformance claim.

## Verification

- Focused path tests prove abbreviated attribute presence and exact literal
  equality use the typed, charged attribute axis.
- The local source-node-variable lifecycle test proves `xsl:value-of`,
  `xsl:for-each`, and `xsl:apply-templates` consume one immutable binding
  without changing modern variable semantics.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
