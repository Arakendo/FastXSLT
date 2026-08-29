# XSLT30 `conflict-resolution-0201` Attribute-Value Pattern

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0201`
- Stylesheet: `conflict-resolution-0201.xsl`
- Environment: case-local embedded principal source

## Method

The metadata-driven apply-templates helper now resolves both named shared
environments and case-local environments from the pinned test-set metadata.
The embedded source, stylesheet, and asserted XML are admitted to a bounded
sealed snapshot and execute as an identified batch of one without ambient I/O.

The compiler extends the existing unnamespaced attribute-presence pattern with
only the exact `element[@attribute='literal']` shape. It retains the expanded
element and attribute names, owned literal value, and non-simple default
priority. Runtime applicability first tests the element name, then charges each
inspected attribute and compares the matching attribute's string value.

A focused compiler control preserves the previously admitted presence form and
the new value-bearing representation. A `!=` predicate remains explicitly
unsupported rather than being approximated as equality or presence.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0201` | `Match-on-node-name,Match-w/qualified-attribute-value` | equal | passed |

The exact `file` rule handles the first input element whose `test` value is
`true`. The higher-priority value predicate handles only the second element
whose value is `false`.

## Claim boundary

This evidence admits only ASCII unnamespaced element and attribute names with
one exact single-quoted string literal. It does not admit namespaces, variables,
double-quoted or escaped XPath literals, numeric comparison, `!=`, general
comparison semantics, boolean expressions, arbitrary predicates, ambiguity
warnings, or the complete 52-case apply-templates denominator.
