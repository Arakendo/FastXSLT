# XSLT30 Include Denominator Inventory

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/include/_include-test-set.xml` |
| Cases | 16 |
| Current ledger | 12 selected/passed; 4 harness-unsupported / not-run as of 2026-08-29 |
| Catalog stylesheet references | 16 principal; 34 secondary (including repeated case environments) |

## Conserved denominator

An executable inventory now fixes the exact ordered names of all 16 cases,
requires one principal stylesheet per case, counts all 34 catalog-declared
secondary stylesheet references, and conserves the direct result shapes as 14
`assert-xml`, one `any-of`, and one expected `error`. A first-party overlay gives
every case an explicit default `harness-unsupported / not-run` disposition,
with first-party selected/passed overrides for `include-0401`, `include-0201`,
`include-0301`, `include-0202`, `include-0105`, `include-0601`, and
`include-0501`, `include-0103`, `include-0104`, `include-0701`, `include-0702a`,
and `include-0702c`. FastXSLT
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
`include-0501` adds two sibling imports and declaration-order precedence between
their competing global parameter defaults.
`include-0103` adds fragmentless acquisition followed by `xml:id` selection,
inherited `xml:base`, and one nested include. The adjacent `include-0102`
remains not-run because its identifier typing depends on a DTD, which the
current XML boundary deliberately denies.
`include-0104` adds the exact leading-import/then-include topology and proves an
included rule retains principal precedence when invoking `xsl:apply-imports`.
`include-0701` adds the sealed five-module two-include/two-leaf-import graph,
file-backed harness inputs, and later same-precedence rule recovery.
`include-0702a` and `include-0702c` execute that graph while conserving the
difference between an explicit XSLT 1.0/2.0 recover request and the XSLT 3.0+
positive case. `include-0702b` remains visibly not-run because its expected
`XTRE0540` outcome requires error-on-multiple-match behavior that the private
compiler and invocation policy cannot request.
It makes no general claim for `xsl:include`, `xsl:import`, import precedence,
general embedded stylesheet fragments, module cycles, or arbitrary module graphs. Upstream
bytes remain immutable in the W3C submodule; all disposition policy remains in
the first-party overlay.
