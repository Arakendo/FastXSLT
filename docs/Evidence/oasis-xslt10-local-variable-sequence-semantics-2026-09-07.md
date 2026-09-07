# OASIS XSLT 1.0 Local Variable and Sequence Semantics

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can childless local bindings and source-node-valued local variables preserve
their XSLT 1.0 value kinds while continuing to use the shared typed compiler,
runtime frames, source XDM, work control, and result construction?

## Change

A childless, untyped local `xsl:variable` with neither `select` nor content now
compiles to an immutable empty atomic string. Non-empty constructors continue
to create temporary trees. This mirrors the already verified global-binding
rule and repairs value-kind identity rather than special-casing a boolean or
string consumer.

Variable-only `xsl:for-each/@select` no longer assumes that every variable is a
temporary-tree document. The private instruction now resolves the existing
runtime value kind and iterates either the temporary-tree document or the
source-node sequence. Source nodes retain their identity and document order;
the loop receives the correct position and size through the ordinary sequence
focus.

For XSLT 1.0 static context, a plain variable `xsl:value-of` now reuses the
existing compatibility-isolated variable string conversion. A source-node set
therefore contributes the string value of its first node in document order,
while modern stylesheets retain their existing sequence/cardinality behavior.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,333 | 1,333 | 0 |
| Executed successfully | 1,160 | 1,172 | +12 |
| Expected-result XML matches | 1,050 | 1,061 | +11 |
| XML comparison mismatches | 75 | 76 | +1 |
| Execution failures | 173 | 161 | -12 |

`Lotus/select_select11#1` is the representative unchanged archival case: a
local `select="human"` node-set is consumed directly by
`xsl:for-each select="$which"` and now produces the expected `Anne,Elissa,`
result. Eleven of the twelve newly executing cases match their archival
expected results.

`Microsoft/Variables__84439#1` is the newly exposed mismatch and receives no
pass credit. Its node-set parameter now uses the correct first-node string
conversion, but its expected source-whitespace behavior differs from the
current preserved source XDM. That separate whitespace obligation remains
visible rather than being folded into this variable-iteration repair.

The childless local-binding correction changes no corpus disposition in this
sweep, but its focused test prevents the local and global value-kind rules from
diverging again.

The strict standard-operation lower bound is now
`1,061 / 2,742 = 38.69%`; the deliberately conservative all-catalog ratio is
`1,061 / 3,173 = 33.44%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Verification

- A focused runtime test proves empty local/global strings remain distinct
  from non-empty constructed temporary trees.
- A focused lifecycle test proves one local source-node variable supports both
  XSLT 1.0 first-node `xsl:value-of` conversion and complete `xsl:for-each`
  iteration.
- The existing modern multi-node `xsl:value-of` test confirms that XSLT 3.0
  does not inherit the XSLT 1.0 first-node conversion.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
