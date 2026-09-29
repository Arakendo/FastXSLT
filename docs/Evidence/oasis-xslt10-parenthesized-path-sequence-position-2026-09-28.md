# OASIS XSLT 1.0 Parenthesized Path and Sequence Position -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Lotus axis and position cases distinguish a predicate on the
last step from a filter over the complete path result:

```xpath
descendant-or-self::*/@att1[last()]
(descendant-or-self::*/@att1)[last()]
(child::chapter/descendant-or-self::node())/footnote[2]
```

FastXSLT already normalized and deduplicated every completed path step in
document order, and already owned the typed `last()` position predicate. The
private path plan did not retain the fact that the final predicate belonged to
the complete path sequence, and the parser rejected a parenthesized path used
as the prefix of a following `/` step.

## Change

`LocationPath` now optionally retains one complete-sequence position predicate.
The initial admitted form is exactly `(path)[last()]`. Runtime applies it only
after the full path result has been ordered and deduplicated, preserving the
different semantics of a step-local `path[last()]` predicate.

A parenthesized, non-union path may also compose with a following ordinary `/`
step. This is a syntax normalization over the existing typed path plan; it does
not introduce an alternate evaluator, change node identity, or bypass work
control. Parenthesized unions, arbitrary filter expressions, completion-order
semantics, and generalized XPath expression grouping remain outside this slice.

## Corpus result

Unchanged `Lotus/position_position89#1` now initializes, executes, and compares
exactly. `Lotus/axes_axes122#1` advances through both admitted parenthesized
forms and now stops at its later, honestly unsupported parenthesized union.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,437 | 2,438 | +1 |
| Executed successfully | 2,389 | 2,390 | +1 |
| Initialization failures | 733 | 732 | -1 |
| Execution failures | 48 | 48 | 0 |
| Exact XML expected-result matches | 2,226 | 2,227 | +1 |
| Standard-operation `XPST0003` initialization frontier | 11 | 8 | -3 |

The conservative exact-match ratio is `2,227 / 3,173 = 70.19%`.
Expected-error credit remains 423 / 431, comparator gaps remain 54, and
successfully executed ordinary XML mismatches remain zero.

## Verification

Focused tests distinguish whole-sequence and step-local `last()` and prove that
a parenthesized multi-step prefix composes with a following positional step.
The unchanged archival case then verifies compilation, execution, serialization,
and comparison through the production workbench path.

```powershell
cargo test -p fastxslt parenthesized_path_ --all-features
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/position_position89#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/axes_axes122#1'
```

## Normative references

- [XPath 1.0 section 2.4, Location Paths](https://www.w3.org/TR/1999/REC-xpath-19991116#location-paths)
- [XPath 1.0 section 3.3, Node-sets](https://www.w3.org/TR/1999/REC-xpath-19991116#node-sets)

