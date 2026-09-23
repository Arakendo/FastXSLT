# OASIS XSLT 1.0 Nested Child Attribute-Prefix Predicate

Date: 2026-09-23  
Status: Verified semantic and compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the shared typed location-path evaluator admit the nested predicate
`Name[starts-with(@First, 'J')]` without adding key-specific runtime behavior or
a general expression fallback?

## Implemented slice

Yes. The typed path boolean plan now represents one unqualified child element,
one unqualified attribute, and one literal `starts-with()` prefix. Evaluation
visits the candidate's children and their attributes in document order, charges
every node visit and string operation, and returns the XPath node-set effective
boolean value: true when at least one matching child has the requested
attribute prefix.

This is shared path semantics. The XSLT 1.0 key-pattern work merely exposes it
inside the already-admitted descendant tail.

## Corpus result

Unchanged `Lotus/idkey_idkey44#1` and `Lotus/idkey_idkey45#1` move from
initialization rejection directly to exact XML comparison passes.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,206 | 2,208 | +2 |
| Executed successfully | 2,153 | 2,155 | +2 |
| Expected-result XML matches | 2,005 | 2,007 | +2 |
| XML comparison mismatches | 64 | 64 | 0 |
| Execution failures | 53 | 53 | 0 |
| Comparator unsupported | 75 | 75 | 0 |

The strict complete-catalog lower bound becomes
`2,007 / 3,173 = 63.25%`.

## Boundaries

- This is one typed nested child/attribute-prefix predicate, not a general
  nested XPath evaluator.
- Names are unqualified in this first slice; namespace-aware expansion needs
  separate static-context evidence.
- Traversal and comparison remain charged and cancellation-observable.
- No resource authority, cache, public API, or XSLT-version-specific runtime
  branch is introduced.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features path_boolean_predicates_select_children_by_nested_attribute_prefix
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/idkey_idkey44#1'
./scripts/verify.ps1
```
