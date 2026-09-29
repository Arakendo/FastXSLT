# OASIS XSLT 1.0 Binary Numeric Sort Key -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Lotus `sort_sort23` case orders source elements with
`xsl:sort select="@height*@width" data-type="number"`. FastXSLT already owned
a typed, charged binary-numeric plan for path operands and XSLT 1.0
first-in-document-order numeric conversion, but the sort-key compiler sent the
same expression to the location-path grammar.

## Change

XSLT 1.0 sort compilation now admits the existing binary-numeric plan before
the location-path fallback. Each candidate evaluates that plan with the sort
candidate's node, position, and size, then passes its lexical result through
the existing numeric-sort conversion and stable ordering path.

This is not a second arithmetic evaluator or a general dynamic sort
expression. Modern XPath typing is unchanged. Work charging, path selection,
numeric conversion, variable ownership, and arithmetic failure behavior remain
owned by the shared binary-numeric evaluator.

## Corpus result

Unchanged `Lotus/sort_sort23#1` initializes, executes, and compares exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,425 | 2,426 | +1 |
| Executed successfully | 2,375 | 2,376 | +1 |
| Initialization failures | 745 | 744 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact XML expected-result matches | 2,214 | 2,215 | +1 |
| Visible comparator gaps | 53 | 53 | 0 |

The conservative exact-match ratio is `2,215 / 3,173 = 69.81%`.
Expected-error credit remains 423 / 431.

## Verification

A focused runtime test proves that three source elements sort by the products
of two attributes through the shared expression plan. The unchanged corpus
case then proves the composed behavior against its archival text result.

```powershell
cargo test -p fastxslt --all-features xslt10_sort_uses_the_shared_binary_numeric_evaluator
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/sort_sort23#1'
./scripts/verify.ps1
```

## Normative references

- [XSLT 1.0 section 10, Sorting](https://www.w3.org/TR/1999/REC-xslt-19991116#sorting)
- [XPath 1.0 section 3.5, Numbers](https://www.w3.org/TR/1999/REC-xpath-19991116#numbers)
