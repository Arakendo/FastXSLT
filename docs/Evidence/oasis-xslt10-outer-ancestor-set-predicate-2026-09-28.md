# OASIS XSLT 1.0 Outer Ancestor-Set Predicate -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Lotus `axes_axes86` case selects collateral branches of every
ancestor that has multiple element children:

```xpath
ancestor::*[count(child::*) > 1]/*
  [not(. = current()/ancestor-or-self::*)]
```

FastXSLT already owned the ancestor and ancestor-or-self axes, intermediate
step predicates, relative element counting, XSLT 1.0 outer-current focus, and
XPath 1.0 node-set string comparison in smaller forms. Two compositional gaps
kept the complete expression outside the typed path grammar.

## Change

Relative element-count predicates now recognize explicit `child::*` as the
same child-element wildcard already represented by `*`. XSLT 1.0 path
normalization also retains the bounded comparison between a predicate
candidate and `current()/ancestor-or-self::*` as a typed outer-focus
predicate.

Execution compares the candidate string value with every element in the
outer current node's ancestor-or-self set, preserving XPath 1.0 general
comparison semantics. It charges the comparison and every visited node. No
node-identity comparison, general primary-expression path, namespace-node
support, or ambient current-node state was introduced.

## Corpus result

Unchanged `Lotus/axes_axes86#1` initializes, executes, and compares exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,426 | 2,427 | +1 |
| Executed successfully | 2,376 | 2,377 | +1 |
| Initialization failures | 744 | 743 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact XML expected-result matches | 2,215 | 2,216 | +1 |
| Generic `FXXP1001` initialization frontier | 11 | 10 | -1 |

The conservative exact-match ratio is `2,216 / 3,173 = 69.84%`.
Expected-error credit remains 423 / 431.

## Verification

A focused path test evaluates the complete compound expression over a small
tree and verifies that only the collateral children survive. The unchanged
corpus case then proves the semantics against the archival expected result.

```powershell
cargo test -p fastxslt --all-features xslt10_outer_context_ancestor_set_composes_with_intermediate_child_count
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/axes_axes86#1'
./scripts/verify.ps1
```

## Normative references

- [XPath 1.0 section 2.4, Location Paths](https://www.w3.org/TR/1999/REC-xpath-19991116#location-paths)
- [XPath 1.0 section 3.4, Booleans](https://www.w3.org/TR/1999/REC-xpath-19991116#booleans)
- [XSLT 1.0 section 12.4, `current()`](https://www.w3.org/TR/1999/REC-xslt-19991116#function-current)
