# OASIS XSLT 1.0 Path-Valued Number Conversion

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged Lotus `numbering79` supplies a relative child path as the `value` of
`xsl:number`. FastXSLT already admitted literals, the context item,
`position()`, and bounded numeric expressions, but rejected an ordinary
node-selection value at `FXXP1022`.

## Implemented slice

Under XSLT 1.0 compatibility, `xsl:number/@value` may now retain a typed
location path. Runtime evaluation:

- evaluates the path through the charged location-path evaluator;
- selects the first node in document order, as required by XPath 1.0
  node-set-to-number conversion;
- obtains its charged string value, or the empty string for an empty selection;
  and
- feeds that lexical value through the existing XSLT 1.0 number conversion and
  formatting policy.

The modern profile is unchanged, and the implementation does not create a
second path evaluator or uncharged traversal.

## Verification and corpus disposition

A focused regression proves first-node selection, numeric rounding, nonnumeric
lexical pass-through, and empty-selection behavior.

Unchanged `Lotus/numbering_numbering79#1` now initializes and executes, removing
the final `FXXP1022` initialization frontier. It remains a visible, uncredited
XML mismatch: the archive expects its discretionary nonnumeric/NaN rendering as
`(0)`, while FastXSLT's established XSLT 1.0 policy passes through the selected
lexical value, which is empty for the missing `wiseguy` child. The corpus case
itself records discretionary `number-not-positive` behavior, so FastXSLT does
not change a coherent compatibility policy merely to match that processor's
choice.

The complete sweep reaches **2,326 initialized** and **2,278 successfully
executed**. The strict lower bound remains **2,111 / 3,173 exact matches
(66.53%)**. Initialization failures fall to 809, visible XML mismatches rise to
82, and execution failures and comparator exclusions remain unchanged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_number_value_converts_the_first_selected_node
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/numbering_numbering79#1'
```
