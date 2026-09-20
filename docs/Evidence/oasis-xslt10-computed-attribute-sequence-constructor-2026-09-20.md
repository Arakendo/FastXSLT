# OASIS XSLT 1.0 Computed-Attribute Sequence Constructor

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 computed attribute execute an ordinary bounded sequence
constructor, while preserving the specification's recovery behavior for
non-text result nodes and the engine's existing execution controls?

## Implemented slice

After the smaller literal, value, number, `for-each`, `copy-of`, and local
count plans have been considered, an XSLT 1.0 computed attribute may retain a
compiled sequence of at most 64 meaningful stylesheet children. The sequence
uses only instructions admitted by the ordinary compiler and executes with the
complete current focus, mode, matched-template identity, call depth, variable
frame, cancellation, and work budgets.

Only top-level text results contribute characters to the attribute value.
Constructed elements, attributes, comments, and processing instructions are
ignored with their content under the bounded XSLT 1.0 recovery rule. Direct
node constructors are omitted during compilation, which also prevents an
ignored `xsl:copy` from recursively expanding `use-attribute-sets`. Dynamic
instructions such as `xsl:apply-templates` and `xsl:call-template` still run
through the ordinary engine; their top-level result nodes are filtered by the
same rule.

The compiled sequence participates in known-capacity accounting and
decimal-format binding traversal. Modern stylesheets remain on their prior
explicit boundary. No alternate evaluator, ambient resource access,
cross-invocation retention, or public representation was introduced.

## Corpus result

The unchanged `Microsoft/Text_ReservedCharsInAttribute#1` case now runs a
`for-each` containing `apply-templates` inside `xsl:attribute` and becomes an
exact XML-semantic match. The remaining eleven former `FXST1033` observations
advance to their distinct next frontiers, including source XML policy,
attribute-set scope, computed comment/processing-instruction content, and
expression breadth; none is silently counted as a pass.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,078 | 2,079 | +1 |
| Initialization failures | 1,057 | 1,056 | -1 |
| Executed successfully | 1,976 | 1,977 | +1 |
| Exact XML-semantic matches | 1,847 | 1,848 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |
| `FXST1033` initialization frontier | 12 | 0 | -12 |

The exact compatibility lower bound becomes `1,848 / 3,173 = 58.24%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- A focused regression combines literal text, a deliberately ignored literal
  result element, `for-each`, and nested `apply-templates`; only the produced
  top-level text contributes to the attribute value.
- The complete conserved sweep advances one unchanged case to exact output and
  exposes every other former `FXST1033` case at its next distinct boundary.
- A preliminary sweep exposed recursive attribute-set expansion beneath an
  ignored `xsl:copy`; the retained compiler recovery rule removes that path and
  the completed sweep terminates normally.
- No upstream corpus byte was edited.
