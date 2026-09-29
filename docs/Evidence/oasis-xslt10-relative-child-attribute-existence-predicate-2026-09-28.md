# OASIS XSLT 1.0 Relative Child-Attribute Existence Predicate -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Microsoft sorting cases use this standard XPath 1.0 sort key:

```xpath
ancestor::*[fruit/@name]
```

FastXSLT already owned the ancestor axis, per-step boolean predicates, bounded
relative child paths, and attribute nodes. The predicate representation could
express several more specialized nested paths but did not retain the ordinary
effective-boolean-value rule for a relative child path ending in an attribute.

## Change

The private predicate plan now admits a bounded relative child path of at most
four unqualified steps. An unqualified attribute step is permitted only at the
end. Runtime evaluates the path from each predicate candidate and uses node-set
non-emptiness as its effective boolean value. Child and attribute visits use the
existing invocation work-control charge points.

No descendant abbreviation, namespace-qualified step, arbitrary predicate
expression, document switch, new node identity, or public XPath representation
was introduced.

## Corpus result

Both unchanged Microsoft `Sorting__89749` and `Sorting__89751` now initialize,
execute, and compare exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,435 | 2,437 | +2 |
| Executed successfully | 2,387 | 2,389 | +2 |
| Initialization failures | 735 | 733 | -2 |
| Execution failures | 48 | 48 | 0 |
| Exact XML expected-result matches | 2,224 | 2,226 | +2 |
| Standard-operation `XPST0003` initialization frontier | 13 | 11 | -2 |

The conservative exact-match ratio is `2,226 / 3,173 = 70.15%`.
Expected-error credit remains 423 / 431 and successfully executed ordinary XML
mismatches remain zero.

## Verification

A focused path test proves that only ancestors having the requested nested
attribute path survive. The unchanged archival cases then verify the expression
as an `xsl:sort` key through production compilation, execution, and comparison.

```powershell
cargo test -p fastxslt axis_predicates_select_relative_child_attribute_path_existence
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Sorting__89749#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Sorting__89751#1'
./scripts/verify.ps1
```

## Normative references

- [XPath 1.0 section 2.4, Location Paths](https://www.w3.org/TR/1999/REC-xpath-19991116#location-paths)
- [XPath 1.0 section 3.3, Node-sets](https://www.w3.org/TR/1999/REC-xpath-19991116#node-sets)
- [XPath 1.0 section 3.4, Booleans](https://www.w3.org/TR/1999/REC-xpath-19991116#booleans)
