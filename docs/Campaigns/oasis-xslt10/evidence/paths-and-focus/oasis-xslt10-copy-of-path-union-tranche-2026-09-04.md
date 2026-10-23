# OASIS XSLT 1.0 Copy-Of Path Union Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 476 definite unchanged XML passes; 708 initialized cases |
| Result | 481 definite unchanged XML passes; 715 initialized cases |
| Disposition | Bounded node-path union for `xsl:copy-of`; not general XPath-union conformance |

## Change

`xsl:copy-of` can now compile a union whose alternatives are admitted location
paths, including qualified child names resolved from stylesheet namespace
bindings. Each alternative executes through the existing charged path evaluator.
The union operator itself is charged, combines the selected node identities,
restores source document order, removes duplicates, and only then invokes the
existing complete source-node copier.

This ordering is semantically important for expressions such as `*|@*`:
attribute nodes must reach result element construction before copied children
even though the child alternative appears first. The focused regression also
selects one namespaced child twice and proves both deduplication and preserved
source namespace nodes. Retained-capacity accounting includes every compiled
alternative, while semantic inspection continues to expose only the stable
`CopyOf` feature rather than private plan layout.

## Measurement

The complete hash-verified local sweep moves as follows:

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 708 | 715 | +7 |
| Execution succeeded | 534 | 540 | +6 |
| XML comparison passes | 476 | 481 | +5 |
| XML comparison mismatches | 29 | 30 | +1 |
| XML comparator gaps | 16 | 16 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **481 / 2,742 = 17.54%** of standard-operation
cases and **481 / 3,173 = 15.16%** of the complete catalog.

One newly initialized case reaches a structured `XTDE0410` result-construction
failure and remains uncredited. `Lotus/copy_copy09#1` executes but joins the
visible mismatch set: its copied namespace and attribute structure is present,
while actual and expected indented XML contain different whitespace text. The
case also carries upstream `generated-namespace-prefix` doubts metadata. This
evidence does not reclassify the mismatch or treat doubts metadata as a pass.

## Boundaries

This tranche does not admit arbitrary XPath union operands, variable unions,
atomic operands, expressions containing unsupported path alternatives, or
unions in other instruction types. Those require a shared expression-level
representation rather than widening this bounded copy construction silently.
