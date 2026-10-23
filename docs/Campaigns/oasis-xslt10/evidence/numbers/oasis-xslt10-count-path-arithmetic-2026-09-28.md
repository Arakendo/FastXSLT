# OASIS XSLT 1.0 Count-Path Arithmetic -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Microsoft miscellaneous case uses two node-set cardinalities as
operands of one XPath 1.0 subtraction:

```xpath
count(//doc/*) - count(following::ref)
```

FastXSLT already owned controlled location-path evaluation, `count(path)` value
production, and a checked exact-rational arithmetic tree. The arithmetic tree
could not yet retain a `count(path)` leaf, so the expression was misrouted into
location-path parsing and reported as invalid syntax.

## Change

In XSLT 1.0 compatibility mode, the private binary-numeric tree now admits
`count(location-path)` as one typed operand. Evaluation uses the existing
controlled path evaluator, converts the selected cardinality to the exact
numeric representation, and charges one additional XPath operation before
composition with the surrounding arithmetic tree.

This does not introduce a second count evaluator, a runtime version branch,
general function operands, dynamic function calls, or relaxed modern XPath
cardinality rules.

## Corpus result

Unchanged `Microsoft/Miscellaneous__84435#1` advances from invalid `XPST0003`
to its next explicit unsupported boundary, variable-valued
`xsl:number/@value` (`FXXP1022`). Its principal source also contains a DTD and
therefore remains outside the current host parser policy. The case receives no
pass credit.

The conserved totals remain 2,228 / 3,173 exact matches (70.22%), 2,439
initialized cases, 2,391 successful executions, 423 / 431 expected-error
credit, 54 comparator gaps, and zero ordinary successfully executed XML
mismatches.

## Verification

A focused compiler regression admits the unchanged lexical form. A production
runtime regression evaluates `count(/doc/*) - count(/doc/b)` through stylesheet
compilation, transformation, and text serialization and obtains `2`.

```powershell
cargo test -p fastxslt --all-features xslt10_binary_numeric_tree_composes_count_operands
cargo test -p fastxslt --all-features compiles_mixed_path_literal_arithmetic_tree
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Miscellaneous__84435#1'
```

## Normative references

- [XPath 1.0 section 3.4, Booleans](https://www.w3.org/TR/1999/REC-xpath-19991116#booleans)
- [XPath 1.0 section 4.1, Node Set Functions](https://www.w3.org/TR/1999/REC-xpath-19991116#function-count)
