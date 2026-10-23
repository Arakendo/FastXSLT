# OASIS XSLT 1.0 Nested Child Boolean Path Predicate

- Date: 2026-09-29
- Status: Verified private XPath and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

`Microsoft/BVTs_bvt100#1` uses one location-path predicate that composes
boolean operators, nested child selections, positional filtering, string
comparison, attribute comparison, and `not()`:

```xpath
bookstore[
  book[author[last-name][2] = 'Kimball']
  or @specialty = 'novel'
     and (not(book[title[1] = 'Unknown']) or book[title[1] = 'Book 1'])
]
```

The existing bounded predicate parser could evaluate the individual scalar
forms, but it could not preserve nested predicate structure inside a child
selection. The general boolean-expression compiler could also mistake an
equality nested inside brackets for a top-level equality.

## Bounded implementation

The private path-predicate representation now admits two compositional forms:

- a child-element selection evaluated with its own ordered focus and a nested
  predicate; and
- a positional child selection whose surviving node's string value is compared
  with a literal.

Top-level equality detection now observes quote, parenthesis, and predicate-
bracket depth. Nested evaluation preserves the original outer context, derives
position and size from the selected child focus, short-circuits boolean
operators, and charges visits where navigation occurs. Existing specialized
predicate forms retain precedence over the generic nested-child form.

This does not select a general XPath AST, arbitrary reverse-axis positional
semantics, or a second evaluator. It extends the existing typed bounded path
predicate representation and keeps the complete location-path evaluator as the
semantic owner.

## Corpus result

The unchanged `Microsoft/BVTs_bvt100#1` case now initializes, executes, and
compares exactly. The conserved sweep changes as follows:

- exact matches rise from 2,286 to **2,287 / 3,173 (72.08%)**;
- initialized cases rise from 2,463 to **2,464**;
- successful executions rise from 2,419 to **2,420**;
- initialization failures fall from 707 to **706**;
- execution failures remain **44**;
- serialization-layout-policy exclusions remain **18**;
- visible mismatches and comparator gaps remain **zero**; and
- expected-error credit remains **423 / 431**.

## Verification

```powershell
cargo test -p fastxslt path_boolean_predicate_compares_dynamic_node_sets_and_positional_children --all-features
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt100#1'
./scripts/verify.ps1
```
