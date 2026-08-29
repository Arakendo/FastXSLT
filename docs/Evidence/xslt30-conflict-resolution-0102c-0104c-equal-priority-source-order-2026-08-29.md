# XSLT30 `conflict-resolution-0102c/0104c` Equal-Priority Source Order

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Cases: `conflict-resolution-0102c` and `conflict-resolution-0104c`
- Stylesheets: `conflict-resolution-0102.xsl` and
  `conflict-resolution-0104.xsl`
- Environment: embedded `conflict-resolution-01` principal source
- Native assertions:
  - `0102c`: `<out>Match-of-wildcard</out>`
  - `0104c`: `<out>Match-of-node-type</out>`

## Method

One metadata-driven helper resolves each selected case, shared embedded source,
stylesheet, and expected XML from the pinned test set. Each source/stylesheet
pair is admitted to its own bounded sealed snapshot and executes as an
identified batch of one without ambient filesystem access after admission.

Both stylesheets apply templates to the same `foo` element and declare an
element wildcard rule and an any-node rule. Those patterns have equal default
priority for this admitted conflict family. The stylesheets reverse declaration
order: `0102c` declares `*` last, while `0104c` declares `node()` last. The
XSLT 3.0 default multiple-match behavior therefore selects the later matching
rule in each case.

The prior private selector accidentally ranked the element wildcard above the
any-node test. Its internal priority bands now preserve path patterns above
exact element/attribute names, exact names above this wildcard/node-test band,
and source-order last-match selection within an equal band. Re-executing
`conflict-resolution-0101` proves the exact `foo` rule still defeats both
fallbacks.

## Result

| Case | Later equal-priority rule | Expected | Actual | Disposition |
| --- | --- | --- | --- | --- |
| `conflict-resolution-0102c` | `*` | `<out>Match-of-wildcard</out>` | semantically equal XML | passed |
| `conflict-resolution-0104c` | `node()` | `<out>Match-of-node-type</out>` | semantically equal XML | passed |

## Claim boundary

This evidence admits only the XSLT 3.0 default last-declared behavior for these
two equal-priority element matches. It does not admit explicit `priority`,
`xsl:mode/@on-multiple-match`, warning delivery, XSLT 1.0/2.0 recovery modes,
import precedence, package precedence, union patterns, namespace wildcards,
predicate ambiguity, or the complete 50-case apply-templates denominator.
