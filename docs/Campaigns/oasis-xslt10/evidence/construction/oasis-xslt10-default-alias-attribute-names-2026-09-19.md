# OASIS XSLT 1.0 Default-Alias Attribute Names

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

A namespace alias whose `stylesheet-prefix` was `#default` incorrectly rewrote
unprefixed literal result attributes into the alias result namespace. This
produced names such as `xsl:version` where the stylesheet constructed the
ordinary no-namespace attribute `version`.

Default element namespaces never place an unprefixed attribute in that
namespace. Namespace-alias rewriting therefore must distinguish an unprefixed
attribute from an element using the default namespace, even though both are
represented with no namespace URI before element-name expansion.

## Repair

- Keep the existing namespace-URI rewrite for literal result element names.
- Rewrite a literal result attribute name only when that attribute already has
  an explicit namespace URI.
- Preserve namespace declaration rewriting and the existing behavior for
  prefixed literal result attributes.
- Add a focused compiler regression containing both a prefixed aliased
  attribute and an unprefixed attribute on the same literal result element.

## Result

The unchanged Microsoft `Namespace-alias__91781#1`,
`Namespace-Alias_Test1#1`, and `Namespace-Alias_Test2#1` cases now match their
expected XML results. The complete 3,173-case sweep remains at 2,026 initialized
and 1,925 successfully executed cases. Exact XML-semantic matches rise to 1,798
and the visible mismatch set falls to 53.

## Boundary conclusion

This corrects expanded-name handling for unprefixed literal result attributes;
it does not change serializer prefix choice, namespace declaration order, or
the still-deferred conflict/import-precedence composition of aliases declared
in different stylesheet modules.
