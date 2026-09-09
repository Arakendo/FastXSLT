# OASIS XSLT 1.0 unsupported-attribute frontier and escaping-default slice -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

What syntax is hidden inside the 129-case generic `FXST1009` initialization
frontier, and can the semantically inert
`xsl:text disable-output-escaping="no"` form advance safely without admitting
disable-output-escaping into the result-tree model?

## Measurement refinement

The local measurement now classifies `FXST1009` by the owning instruction and
the unsupported expanded attribute name. The three largest families before
the semantic change were:

| First-failure family | Cases |
| --- | ---: |
| `xsl:attribute/@namespace` | 32 |
| `xsl:sort/@lang` | 30 |
| `xsl:text/@disable-output-escaping` | 16 |

Smaller families remain individually visible rather than disappearing into a
single code-shaped bucket. This changes reporting resolution only; it does not
change case disposition.

## Semantic slice

- `xsl:text disable-output-escaping="no"` compiles as ordinary text because it
  requests the engine's existing escaping behavior.
- `disable-output-escaping="yes"` remains explicitly unsupported under
  `FXST1060`; the semantic result tree is not marked with serializer-bypassing
  state.
- Any other lexical value is invalid under `XTSE0020` rather than silently
  interpreted.

Microsoft `Elements__78362` advances past its `xsl:text` instruction to the
independent computed-comment-content frontier. The other first-failing cases
request `yes`, so the overall initialized, executed, match, mismatch, and
execution-failure counters remain unchanged at 1,525, 1,359, 1,244, 89, and
166 respectively. No compatibility credit is claimed for later-frontier
movement.

## Boundaries

This tranche does not implement disable-output-escaping, add lexical escape
flags to result text, change serialization, or infer that legacy serializer
recovery is supported. It does not select `xsl:attribute/@namespace` or
`xsl:sort/@lang`; the refined inventory makes those separate candidate
families for later evidence-led work.

## Verification

- A focused compiler test proves `no`, `yes`, and invalid lexical behavior.
- A focused measurement-classification test proves instruction/attribute
  ownership survives normalization.
- The unchanged Microsoft case reaches its next explicit initialization
  frontier.
- The complete 3,173-case local measurement preserves every aggregate counter.
- The ordinary workspace verification gate passes.
