# XSLT30 `conflict-resolution-0703` Stylesheet Default Namespace

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0703`
- Stylesheet: `conflict-resolution-0703.xsl`
- Environment: shared `conflict-resolution-07` principal source

## Method

The metadata-driven helper executes the pinned source and stylesheet through a
bounded sealed snapshot and identified batch of one. The compiler inherits the
stylesheet-wide `xpath-default-namespace` into descendant static contexts and
lowers simple unprefixed element patterns and child selections to expanded
names in `http://some.uri/`.

The same default does not apply to unprefixed attribute names. The `@test`
selection and matching attribute pattern remain explicitly in no namespace.
Compiled-state controls repeat this distinction with a separate in-memory
stylesheet before the upstream case is executed.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0703` | `foo"true"` | equal | passed |

The qualified `doc` and `foo` elements match through the inherited default
element namespace, and the unqualified `test` attribute remains selectable and
dispatchable through the attribute rule.

## Claim boundary

This evidence covers stylesheet-wide inheritance for simple unprefixed element
patterns and child selections plus the required no-default-namespace rule for
unprefixed attributes. It does not admit general default-namespaced path
expressions, default-namespace reset behavior, URI validation, arbitrary
pattern grammar, or the adjacent current-mode cases.
