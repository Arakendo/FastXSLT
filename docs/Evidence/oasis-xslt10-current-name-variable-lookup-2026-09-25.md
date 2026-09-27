# OASIS XSLT 1.0 Outer-Current Name Variable Lookup

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Two duplicate Microsoft catalog entries bind a local node-set variable from an
absolute path whose predicate compares a candidate attribute to
`name(current())`. FastXSLT previously sent the complete expression to an
unrelated atomic-cast fallback and reported unsupported `FXXP1008`.

## Implemented slice

The XSLT 1.0 compiler now recognizes a bounded source-node variable selection
of the form `absolute-path[@attribute = name(current())]`. It retains the base
path and unnamespaced attribute name in a typed private instruction. At
execution, the absolute path is evaluated through the existing controlled path
evaluator, then the predicate compares each candidate attribute to the lexical
name of the outer instruction focus.

The variable remains a source-node sequence. Attribute visits and the
predicate operation are charged. The implementation does not reinterpret the
predicate candidate as `current()`, add a general dynamic-function evaluator,
or widen modern XPath with the XSLT-only `current()` function.

## Verification and corpus disposition

A focused regression iterates two differently named source elements and proves
that each local variable selects the correspondingly named lookup record. This
guards the outer-current rule rather than merely proving one empty selection.

The unchanged cases
`Microsoft/XSLTFunctions_CurrentFunctionInVariable#1` and
`Microsoft/XSLTFunctions_DocumentFunctionAbsArgument#1` now initialize,
execute, and compare exactly. The complete sweep moves from **2,319 to 2,321
initialized**, **2,271 to 2,273 successfully executed**, and **2,105 to 2,107
exact matches**. The conserved lower bound is therefore **2,107 / 3,173
(66.40%)**. Initialization failures fall from 816 to 814; execution failures,
XML mismatches, and comparator exclusions remain unchanged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_local_node_set_predicate_compares_attribute_to_outer_current_name
./scripts/measure-oasis-xslt10.ps1
```
