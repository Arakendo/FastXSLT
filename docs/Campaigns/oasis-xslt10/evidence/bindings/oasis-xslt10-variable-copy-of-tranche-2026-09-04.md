# OASIS XSLT 1.0 Variable Copy-Of Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 468 definite unchanged XML passes; 700 initialized cases |
| Result | 476 definite unchanged XML passes; 708 initialized cases |
| Disposition | Shared bounded variable-value construction; not a conformance claim |

## Change

The compiler now recognizes an NCName variable reference in `xsl:copy-of` and
retains a typed operation containing the variable name and stylesheet source
location. Runtime lookup observes the existing local/global binding precedence
and copies every already-admitted value representation:

- one atomic value becomes charged result text;
- an atomic sequence uses the modern sequence-construction rule that separates
  adjacent atomic values with one space;
- source node selections use the existing complete source-node deep copier;
- temporary trees use a private complete deep copier that preserves element
  names, namespaces, attributes, text, comments, processing instructions, and
  child order without applying templates or mutating prepared state; and
- an admitted empty sequence produces no result nodes.

The variable lookup charges XPath work. All produced nodes and text continue
through existing result-node and result-byte budgets, and an unbound variable
reports structured `FXRT0002` at the originating `xsl:copy-of` location.
Retained-capacity accounting and semantic inspection include the new operation.

The focused production-path regression copies a global atomic value, a local
source-node sequence, and a local temporary tree with a namespace and attribute
through one stylesheet. This is one modern runtime path; no XSLT 1.0-only
result-tree representation or evaluator was introduced.

## Measurement

The complete hash-verified local sweep moves as follows:

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 700 | 708 | +8 |
| Execution succeeded | 526 | 534 | +8 |
| XML comparison passes | 468 | 476 | +8 |
| XML comparison mismatches | 29 | 29 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **476 / 2,742 = 17.36%** of standard-operation
cases and **476 / 3,173 = 15.00%** of the complete catalog. Every case newly
admitted by this tranche reaches a definite XML comparison pass; none stops at
a later runtime or comparison frontier.

## Boundaries

This tranche does not admit general variable expressions, variable-relative
paths, node-set unions, result-tree-fragment conversion functions, or ambient
resource acquisition. It copies value kinds already represented by the shared
runtime and does not make the private temporary-tree representation public.
