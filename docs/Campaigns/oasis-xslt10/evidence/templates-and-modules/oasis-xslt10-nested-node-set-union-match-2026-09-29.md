# OASIS XSLT 1.0 Nested Node-Set Union Match Predicate

- Date: 2026-09-29
- Status: Verified private compiler/runtime and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

`Microsoft/ConflictResolution__77881#1` uses one template pattern containing a
top-level pattern union and a nested XPath union inside a predicate:

```xpath
book[(title | @style) | price = 'textbook'] | book/title
```

The template-rule compiler previously split every `|` byte, including those
inside the predicate. Its diagnostic therefore exposed a mangled first
alternative rather than the intact unsupported predicate.

## Bounded implementation

Template-rule decomposition now reuses the quote-, parenthesis-, and
bracket-aware top-level union splitter. An inner `|` cannot become a template
alternative.

The intact first alternative compiles to one private match plan for equality
between a bounded union of unqualified child/attribute node tests and one
string literal. Execution applies XPath 1.0 node-set/string comparison: the
predicate succeeds when any selected node has the requested string value.
Child names use the effective XPath default namespace; unqualified attributes
remain in no namespace. Source and temporary-tree evaluators preserve stored
order and charge each inspected node and controlled temporary string-value
traversal.

This slice does not admit arbitrary path operands, prefixed union members,
computed literals, general comparison operators, or general XPath evaluation
during template selection.

## Corpus result

The unchanged case now compares exactly. The conserved sweep changes as
follows:

- exact matches rise from 2,284 to **2,285 / 3,173 (72.01%)**;
- initialized cases rise from 2,460 to **2,461**;
- successful executions rise from 2,416 to **2,417**;
- initialization failures fall from 710 to **709**;
- execution failures remain **44**;
- visible mismatches and comparator gaps remain **zero**; and
- expected-error credit remains **423 / 431**.

Only the two `id('b')` patterns remain in the standard-operation unsupported
template-pattern frontier; their source documents depend on DTD-typed IDs and
therefore intersect the deliberate parser-authority boundary.

## Verification

```powershell
cargo test -p fastxslt template_union_splitter_preserves_a_nested_node_set_union_predicate --all-features
cargo test -p fastxslt xslt10_match_predicate_compares_a_child_attribute_union_to_a_string --all-features
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/ConflictResolution__77881#1'
./scripts/verify.ps1
```
