# XSLT30 `conflict-resolution-1401` Temporary-Tree Next-Match

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1401`

## Executable behavior

The unmodified upstream stylesheet constructs an invocation-owned temporary
tree containing book- and chapter-level `db:title` elements. Its exact
qualified path selects only the chapter title in mode `m:titlepage-mode`.

An explicit-priority union rule matches that temporary node, constructs the
XHTML `h2`, and invokes `xsl:next-match`. Continuation preserves temporary
focus, current mode, and the current matched-template index. The selector then
chooses the lower-ranked exact `db:title` rule using the same import precedence,
compiled priority, and declaration-order model used for source nodes. That rule
applies the temporary text child, producing XML-equivalent output:

```xml
<h2 xmlns="http://www.w3.org/1999/xhtml">ChapterTitle</h2>
```

The metadata-driven test loads the native source, stylesheet, and expected
result from the pinned suite through sealed resources. It also asserts the
compiled matched-template count, so a silent harness rewrite cannot replace
the upstream rule structure.

## Denominator consequence

The complete 50-case apply-templates ledger now contains 48 selected passes
and 2 visible default not-run dispositions. The remaining cases are `1301` and
schema-aware `1402`.

## Claim boundary

This is one exact standards case, not broad temporary-tree or mode conformance.
It does not admit arbitrary temporary XPath, general union operands, temporary
attributes/comments/processing instructions, schema-aware matching, packages,
or a second execution backend. Temporary and source focus share compiled
template semantics while retaining representation-owned navigation.
