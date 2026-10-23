# XSLT30 `for-003` Focus-Preserving Empty-Sum Execution

Date: 2026-08-26

## Native case

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/for/_for-test-set.xml`
- Case: `for-003`
- Dependency: `XSLT20+`
- Environment/source: `for03` / `for03.xml`
- Stylesheet: `for-003.xsl`
- Assertion: inline `<out>0</out>`

The native expression is intentionally the focus-sensitive form:

```xpath
sum(for $i in order-item return @price * @qty)
```

## Semantic result

The `for` clause binds each `order-item` to `$i`, but it does not replace the
outer focus. Both unqualified attribute steps therefore remain relative to the
matched `order` element. That element has neither `price` nor `qty`, so each
multiplication returns an empty sequence. Concatenating those empty return
sequences remains empty, and `sum(())` returns integer zero.

The native source and stylesheet execute through the shared bounded snapshot
and principal-source invocation path. The serialized result is exactly
`<out>0</out>` after the normal XML declaration, matching the upstream inline
assertion.

## Focus and work verification

A focused test uses bound child items that do carry `price` and `qty`, while the
outer `order` does not. It still returns zero, preventing an implementation from
silently changing the context item to the bound variable. Tuple iteration and
the final sum consume the `xpath-operation` domain; source navigation retains
the separate XPath node-visit domain.

A second case puts both attributes on the outer focus. The private evaluator
then fails as unsupported instead of performing numeric multiplication. This
guards the claim boundary: `for-003` proves empty-sequence and focus behavior,
not decimal arithmetic.

## Conservation and claim boundary

At this checkpoint, the complete `expr/for` denominator was three passed and
one engine-unsupported. General focus manipulation, variables in path
expressions, numeric multiplication, decimal values, non-empty `sum()`
aggregation, and formatting remained outside this evidence. Native `for-004`
subsequently advanced those separate semantics without weakening this focus
rule; see [the complete-denominator evidence](xslt30-for-004-exact-decimal-and-complete-for-denominator-2026-08-26.md).
