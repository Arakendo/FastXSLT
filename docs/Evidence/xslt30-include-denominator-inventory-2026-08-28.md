# XSLT30 Include Denominator Inventory

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/include/_include-test-set.xml` |
| Cases | 16 |
| Current ledger | 0 selected; 16 harness-unsupported / not-run |
| Catalog stylesheet references | 16 principal; 34 secondary (including repeated case environments) |

## Conserved denominator

An executable inventory now fixes the exact ordered names of all 16 cases,
requires one principal stylesheet per case, counts all 34 catalog-declared
secondary stylesheet references, and conserves the direct result shapes as 14
`assert-xml`, one `any-of`, and one expected `error`. A first-party overlay gives
every case an explicit `harness-unsupported / not-run` disposition. FastXSLT
therefore records the complete denominator without calling unresolved module
assembly an engine failure or quietly selecting only easy cases.

Fifteen cases declare at least one secondary stylesheet. `include-0201` is the
exception: despite belonging to this set, it primarily tests `xsl:apply-imports`
against built-in rules and does not provide the resource-resolution pressure
needed for the first module case. Several other cases combine inclusion with
import precedence, `xsl:apply-imports`, multiple-match policy, serialization,
or embedded stylesheet fragments.

## First candidate

`include-0401` is the narrowest useful candidate for the resource boundary. It
has one principal stylesheet, one relative `xsl:include` reference to
`include-0401a.xsl`, an inline source, and an `assert-xml` result. Its secondary
module is a simplified stylesheet that reads one global variable declared by
the principal module and produces `<out><in>Hi there!</in></out>`.

The candidate is intentionally not selected yet. Executing it requires all of:

- base identity plus relative-reference resolution over sealed resources;
- bounded acquisition of exactly one secondary module;
- standards-correct include assembly rather than textual XML concatenation;
- simplified stylesheet compilation;
- global-variable visibility across the included module; and
- exact result comparison through the existing in-memory runtime.

Those requirements make it a small vertical dependency case, not merely a URI
parser test. AR-0014 owns the unresolved reference/authority composition, while
the compiler continues to own module semantics.

## Claim boundary

This inventory proves pinned metadata conservation only. It makes no claim that
FastXSLT implements `xsl:include`, `xsl:import`, import precedence, embedded
stylesheet fragments, module cycles, or relative URI resolution. Upstream bytes
remain immutable in the W3C submodule; all disposition policy remains in the
first-party overlay.
