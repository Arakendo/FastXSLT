# OASIS XSLT 1.0 Deep Import Precedence

- Date: 2026-09-24
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The module compiler preserved one nested import level but rejected an imported
program once its compiled template precedences extended below `-1`. This was a
private implementation restriction rather than an XSLT semantic boundary. It
blocked five unchanged OASIS cases containing three- and four-level import
chains, including chains that exercise `xsl:apply-imports` and relative URI
resolution at every module's own base identity.

The neighboring two-sibling-import composition also assigned fixed precedence
shifts. Those constants were sufficient for the previously admitted shallow
branches, but deeper branches could make the two precedence bands overlap.

## Implemented slice

Imported programs may now retain an arbitrary host-bounded depth of already
compiled import precedence. Rebasing preflights every matched-template,
`xsl:apply-imports` floor, root-template floor, and attribute-set precedence
with checked integer arithmetic before mutating the program.

Two sibling imported programs now receive disjoint precedence bands derived
from their actual compiled ranges. The later import remains above the complete
earlier branch, while every branch retains its internal precedence order and
its own `xsl:apply-imports` lower bound.

This changes neither resource authority nor dependency admission. Depth,
module count, admitted bytes, and resolution attempts remain bounded by the
existing host-supplied dependency policy; compilation still consumes only the
sealed resource graph.

## Verification

Focused runtime regressions prove that:

- `xsl:apply-imports` traverses four compiled precedence levels in order; and
- a later deep sibling import cannot enter the earlier sibling's precedence
  band, even when that earlier branch is deeper.

The unchanged OASIS cases below now initialize, execute, and compare exactly:

- `Lotus/impincl_impincl19#1`;
- `Lotus/impincl_impincl26#1`;
- `Lotus/reluri_reluri01#1`;
- `Lotus/reluri_reluri02#1`; and
- `Lotus/reluri_reluri03#1`.

The full sweep moves from **2,311 to 2,316 initialized**, **2,259 to 2,264
successfully executed**, and **2,097 to 2,102 / 3,173 exact matches (66.25%)**.
Initialization failures fall from 824 to 819; execution failures and visible
XML mismatches remain unchanged at 52 and 77 respectively.

## Reproduction

```powershell
cargo test -p fastxslt --all-features apply_imports_traverses_a_four_level_import_chain
cargo test -p fastxslt --all-features deep_sibling_import_branches_keep_disjoint_precedence_bands
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'FXST1030'
```
