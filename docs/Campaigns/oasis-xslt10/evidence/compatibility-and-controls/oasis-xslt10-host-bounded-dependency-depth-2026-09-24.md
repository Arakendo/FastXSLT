# OASIS XSLT 1.0 Host-Bounded Dependency Depth

- Date: 2026-09-24
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related reviews: AR-0014 and AR-0019

## Pressure

The local OASIS importer already acquired transitive stylesheet modules into a
sealed snapshot, but compilation still used hidden private ceilings of two
dependency edges, five modules, one MiB of dependency bytes, and five resolver
attempts. Twenty-one unchanged cases therefore stopped at `FXRS0006` before
their already implemented graph shapes could be assessed.

## Implemented slice

The unstable workbench limit envelope now carries four independent
stylesheet-compilation bounds supplied by its host:

- maximum dependency depth;
- maximum module occurrences, including the principal module;
- maximum aggregate dependency bytes; and
- maximum sealed-snapshot resolution attempts.

The dependency loader and snapshot resolver enforce those values during
compilation. A focused four-module include-chain regression proves that a depth
bound of two rejects the third edge with structured `FXRS0006 / limit`, while a
host-supplied bound of three admits the same sealed graph and transforms it.

The ordinary workbench defaults remain the previous narrow `2 / 5 / 1 MiB / 5`
envelope. Only the hash-verified OASIS measurement host selects
`8 / 64 / 8 MiB / 64`, matching its independently bounded transitive-discovery
envelope. No engine default, ambient authority, or filesystem access was
widened.

## Corpus result

All 21 former limit cases receive a later explicit disposition. Eight become
exact expected-result matches:

- `Lotus/reluri_reluri04#1` through `Lotus/reluri_reluri08#1`;
- `Microsoft/BVTs_bvt040#1`;
- `Microsoft/Include_RelUriTest2#1`; and
- `Microsoft/Include_RelUriTest4#1`.

The other thirteen remain visible at independent semantic or graph-shape
boundaries, including unsupported nested-import precedence, branching graph
composition, output-property composition, path expressions, and the existing
two-direct-dependency compiler slice. `Microsoft/Errors_err053#1` now reaches
the existing cycle diagnostic instead of the depth ceiling. No case receives
pass credit merely for moving beyond `FXRS0006`.

The complete sweep moves from:

- 2,278 to 2,286 initialized cases;
- 2,226 to 2,234 successfully executed cases; and
- 2,066 to 2,074 exact expected-result matches.

The exact compatibility lower bound is therefore
**2,074 / 3,173 (65.36%)**. Initialization failures fall from 857 to 849;
execution failures and comparison mismatches remain 52 and 77 respectively.

## Non-claims

This tranche does not select public resolver limits, support arbitrary
dependency graphs, increase direct dependency fanout, provide live callbacks,
or turn corpus-local acquisition into engine authority. It is compatibility
evidence from a locally acquired archive, not an XSLT 1.0 conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features host_supplied_stylesheet_depth_limit_controls_a_sealed_dependency_chain
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXRS0006
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
