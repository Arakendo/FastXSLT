# XSLT30 built-in-template constructed parameter

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 case `built-in-templates-0202` now passes through the
normal sealed-resource, compilation, transform-set, and XML-comparison path.

The case carries two values through recursive built-in document and element
rules:

- an untyped sequence constructor remains an invocation-owned temporary tree;
- the same literal constructor declared `as="xs:string"` is atomized to one
  string before parameter binding.

The explicit descendant template then proves that the first value remains a
node tree, the second is an `xs:string`, and the temporary tree retains its
document/element/text structure for the bounded `$variable//name()` projection.

## Scope

This slice admits only:

- `xs:string` and `xs:string?` on constructed `xsl:with-param` values;
- exact variable `instance of xs:string`; and
- the exact temporary-tree `$variable//name()` projection used by the case.

It does not select a general sequence-type system, arbitrary path expressions
over temporary trees, schema-aware processing, or public temporary-tree types.
Materialization and traversal remain invocation-owned and work-charged.

## Accounting

The conserved six-case `misc/built-in-templates` denominator advances from
three to four passed cases. The two schema-annotation cases remain visibly not
run. Across the conserved XSLT30 denominators this produces 490 passes, 12
engine-unsupported cases, 2,878 profile exclusions, and 118 visible defaults
out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt30_built_in_templates_tests --all-features`

