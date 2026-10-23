# OASIS XSLT 1.0 Grouped Path Union Suffix -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Lotus axis case selects through a path suffix applied to a
parenthesized union:

```xpath
(preceding-sibling::* | following-sibling::*)/ancestor::*[last()]/*[last()]
```

FastXSLT already owned both axes, reverse-axis proximity positions, ordinary
path composition, bounded path unions, document-order normalization, and
identity deduplication. The apply-selection compiler recognized only a union at
the top expression level, so it misclassified this standard form as invalid
axis syntax.

## Change

The XSLT 1.0 apply-selection compiler now admits a parenthesized union of at
most eight location paths followed by one common ordinary path suffix. It
distributes the suffix into private typed alternatives, then reuses the existing
controlled path-union evaluator. Final selection is normalized in document
order and deduplicated exactly as for an ordinary union.

No variable or function union arm, descendant separator at the grouping
boundary, completion-order behavior, public union representation, or general
primary-expression grammar was introduced.

## Corpus result

Unchanged `Lotus/axes_axes91#1` now initializes, executes, and compares exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,438 | 2,439 | +1 |
| Executed successfully | 2,390 | 2,391 | +1 |
| Initialization failures | 732 | 731 | -1 |
| Execution failures | 48 | 48 | 0 |
| Exact XML expected-result matches | 2,227 | 2,228 | +1 |

The conservative exact-match ratio is `2,228 / 3,173 = 70.22%`.
Expected-error credit remains 423 / 431, comparator gaps remain 54, and
successfully executed ordinary XML mismatches remain zero.

## Verification

A focused runtime test proves that two sibling-axis arms compose the shared
reverse-axis suffix, select the same final node, and deduplicate it before
template dispatch. The unchanged archival case verifies the full production
path.

```powershell
cargo test -p fastxslt grouped_apply_path_union_composes_one_common_suffix --all-features
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/axes_axes91#1'
```

## Normative references

- [XPath 1.0 section 2.4, Location Paths](https://www.w3.org/TR/1999/REC-xpath-19991116#location-paths)
- [XPath 1.0 section 3.3, Node-sets](https://www.w3.org/TR/1999/REC-xpath-19991116#node-sets)

