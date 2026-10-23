# OASIS XSLT 1.0 Parent Name Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 525 definite unchanged XML passes; 790 initialized cases |
| Result | 527 definite unchanged XML passes; 792 initialized cases |
| Disposition | Bounded path/function composition; not a conformance claim |

## Change

`xsl:value-of select="name(..)"` now compiles to a node-name value operation
that owns the existing typed parent location path. The parent step is statically
zero-or-one, so this slice has the same result under the XSLT 1.0 first-node
rule and the modern zero-or-one `fn:name` contract; it does not prematurely
select general multi-node compatibility behavior.

Runtime evaluation uses the charged location-path evaluator, charges the
selected node-name inspection, and appends through the bounded result-text path.
An absent parent produces the empty string. A namespaced node whose lexical
prefix cannot be reconstructed remains the existing explicit `FXRT1008`
unsupported boundary rather than receiving a fabricated QName.

The focused end-to-end regression selects an `item` beneath `group` and proves
that the operation produces the parent name through the production runtime.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 790 | 792 | +2 |
| Execution succeeded | 603 | 605 | +2 |
| XML comparison passes | 525 | 527 | +2 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **527 / 2,742 = 19.22%** of standard-operation
cases and **527 / 3,173 = 16.61%** of the complete catalog. Both newly
initialized cases reach definite unchanged XML comparison passes.

## Boundaries

This tranche does not admit arbitrary `name(path)` expressions, choose XPath
1.0 multi-node conversion behavior, reconstruct namespace prefixes, or add a
second path evaluator. Other `name(..)` occurrences behind keys, sorting,
predicates, or unrelated instructions remain uncredited.
