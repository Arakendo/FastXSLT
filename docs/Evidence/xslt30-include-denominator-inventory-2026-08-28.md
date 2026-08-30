# XSLT30 Include Denominator Inventory

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/include/_include-test-set.xml` |
| Cases | 16 |
| Current ledger | 6 selected/passed; 10 harness-unsupported / not-run as of 2026-08-29 |
| Catalog stylesheet references | 16 principal; 34 secondary (including repeated case environments) |

## Conserved denominator

An executable inventory now fixes the exact ordered names of all 16 cases,
requires one principal stylesheet per case, counts all 34 catalog-declared
secondary stylesheet references, and conserves the direct result shapes as 14
`assert-xml`, one `any-of`, and one expected `error`. A first-party overlay gives
every case an explicit default `harness-unsupported / not-run` disposition,
with first-party selected/passed overrides for `include-0401`, `include-0201`,
`include-0301`, `include-0202`, `include-0105`, and `include-0601`. FastXSLT
therefore records the complete denominator without calling unresolved module
semantics an engine failure or quietly dropping cases.

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

The candidate now executes through all of:

- base identity plus relative-reference resolution over sealed resources;
- bounded acquisition of exactly one secondary module;
- standards-correct include assembly rather than textual XML concatenation;
- simplified stylesheet compilation;
- global-variable visibility across the included module; and
- exact result comparison through the existing in-memory runtime.

The exact result is `<out><in>Hi there!</in></out>` after the harness removes an
optional XML declaration from both values. The included implicit root template
retains the secondary module identity, while the referenced `$greeting` binding
retains its principal-module value. See
[XSLT30 include-0401 sealed module execution](xslt30-include-0401-sealed-module-execution-2026-08-28.md).

## Claim boundary

The inventory plus selected execution proves only the named one-include slice,
the independent no-module `include-0201` built-in fallback, and one sealed
single-import/repeated-apply-imports path from `include-0301`.
The adjacent `include-0202` adds parameter transfer to the lower-precedence
rule and one bounded computed result attribute.
`include-0105` independently adds an imported named template plus one principal
global binding that shadows its imported same-named declaration.
`include-0601` adds an imported simplified stylesheet whose implicit template
is normalized as a lower-precedence document rule, plus principal text-rule
`xsl:apply-imports` fallback to the built-in text rule.
It makes no general claim for `xsl:include`, `xsl:import`, import precedence,
embedded stylesheet fragments, module cycles, or multi-module graphs. Upstream
bytes remain immutable in the W3C submodule; all disposition policy remains in
the first-party overlay.
