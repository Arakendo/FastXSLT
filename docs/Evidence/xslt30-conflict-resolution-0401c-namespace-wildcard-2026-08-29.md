# XSLT30 `conflict-resolution-0401c` Namespace Wildcard

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0401c`
- Stylesheet: `conflict-resolution-0401.xsl`
- Environment: shared `conflict-resolution-04` principal source
- Dependency: `spec=XSLT30+`

## Method

The metadata-driven helper resolves the shared environment, stylesheet, and
asserted XML from the pinned suite, then executes the admitted bytes through a
bounded sealed snapshot and identified batch of one.

The pattern compiler resolves the static `bar` binding on both `bar:foo` and
`bar:*`. The exact form lowers to the existing expanded-name representation;
the wildcard lowers to namespace-only element applicability. Both rules retain
their explicit integer priority of `5`, so the XSLT 3.0 selector chooses the
later declaration when both apply to the source element.

Focused controls prove that an unbound match-pattern prefix is invalid. They
also reject `bar:*` without explicit priority: its standard default priority
requires a quarter step, while the current private exact comparison domain
stores doubled priorities and therefore cannot represent that value without a
deliberate widening.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0401c` | `matched bar:*` | equal | passed |

The two compiled rules both match `{http://bar.com/}foo` at priority `5`; the
later namespace-wildcard rule produces the asserted `b` result.

## Claim boundary

This evidence admits ASCII prefixed element QName patterns and namespace
wildcards only when the prefix is statically bound. Namespace wildcards still
require explicit bounded integer priority. This does not admit their implicit
quarter-step priority, local-name wildcards, wildcard attributes, EQNames,
general QName grammar, XSLT 2.0 recover/error behavior, ambiguity warnings, or
the complete 52-case apply-templates denominator.
