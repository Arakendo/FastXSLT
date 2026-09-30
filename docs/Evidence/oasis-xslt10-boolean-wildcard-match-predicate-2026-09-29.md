# OASIS XSLT 1.0 Boolean Wildcard Match Predicate

- Date: 2026-09-29
- Status: Verified private compiler/runtime and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged `Lotus/impincl_impincl16#1` imports a template whose wildcard match
predicate composes XSLT 1.0 boolean and focus semantics:

```xpath
*[(not(.=117) and ((position() > 225) and (position() < 375)))
  and ((@century='yes') or (@foo='nope'))]
```

The existing private match compiler admitted isolated numeric, attribute, and
sequential-position predicate families but rejected this composed expression
with `FXST1005`.

## Bounded implementation

The compiler now recognizes one private wildcard-predicate AST containing:

- nested `and`, `or`, and `not`;
- integer comparison against the context node's XPath string value;
- integer `position()` comparison; and
- equality between one unqualified attribute and one string literal.

The parser observes parentheses and string-literal boundaries and rejects every
other expression form. Runtime evaluation short-circuits boolean operators,
computes wildcard position among element siblings, and charges inspected
siblings, attributes, string-value traversal, and predicate operations through
the existing invocation control. Source and temporary trees share the scalar
predicate evaluator and retain representation-owned navigation.

No general XPath evaluator is invoked during template selection. Prefixed
attributes, variables, functions other than `position()`/`not()`, arithmetic,
non-integer numeric literals, and named-node boolean predicate patterns remain
outside this slice.

## Corpus result

`Lotus/impincl_impincl16#1` now initializes, executes, and compares exactly.
The conserved sweep changes as follows:

- exact matches rise from 2,283 to **2,284 / 3,173 (71.98%)**;
- initialized cases rise from 2,459 to **2,460**;
- successful executions rise from 2,415 to **2,416**;
- initialization failures fall from 711 to **710**;
- execution failures remain **44**;
- visible mismatches and comparator gaps remain **zero**; and
- expected-error credit remains **423 / 431**.

The standard-operation `unsupported template match pattern` frontier falls
from four cases to three. The remaining cases require distinct `id()` or
broader union-pattern semantics and are not implied by this slice.

## Verification

```powershell
cargo test -p fastxslt match_boolean_predicate --all-features
cargo test -p fastxslt xslt10_boolean_wildcard_match_predicate_preserves_focus_and_short_circuiting --all-features
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/impincl_impincl16#1'
./scripts/verify.ps1
```
