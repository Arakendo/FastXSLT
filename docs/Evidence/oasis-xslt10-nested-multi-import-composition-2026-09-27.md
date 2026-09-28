# OASIS XSLT 1.0 Nested Multi-Import Composition -- 2026-09-27

Date: 2026-09-27  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Three unchanged Microsoft BVT cases reached the private `FXST1027` boundary
because an included or imported module contained four or more sibling
`xsl:import` declarations. FastXSLT could already compose one or two compiled
import branches and arbitrary include-only branches, but its compiled-program
import helper encoded the sibling count in its type.

Removing that shape limit exposed two downstream omissions rather than making
them disappear behind graph handling:

- imported output declarations needed ordinary property precedence plus
  cumulative `cdata-section-elements`; and
- named templates needed to retain import precedence when an already-composed
  imported program was later included into a higher-precedence module.

## Change

The private module composer now accepts an ordered non-empty list of two or
more compiled import programs. It rebases branches from right to left into
disjoint precedence bands, preserving later-import precedence and each
branch's `xsl:apply-imports` floor. The homogeneous graph walker uses that
primitive for arbitrary all-import sibling branches; mixed include/import
sibling composition remains outside this slice.

Output-property composition now:

- takes ordinary scalar properties from the highest-precedence declaration;
- admits inherited `method`, `encoding`, `indent`, and
  `omit-xml-declaration` through multiple import branches; and
- unions `cdata-section-elements` across import precedence and subsequent
  same-precedence inclusion, as required by its cumulative semantics.

Compiled named templates now retain a private scalar import precedence.
Import rebasing updates it, imported duplicates select the higher-precedence
declaration, and include composition rejects only duplicates at the same
precedence. This does not change runtime named-template lookup or expose
compiler representation publicly.

A focused sealed-resource regression composes four repeated imports beneath an
include. It proves principal named-template override, cumulative imported and
principal CDATA declarations, and that `xsl:apply-imports` does not cross into
an earlier sibling import subtree.

## Corpus result

All three unchanged cases that previously stopped at `FXST1027` now compile,
execute, and compare exactly:

- `Microsoft/BVTs_bvt041#1` -- repeated imported output declarations and
  cumulative CDATA element names;
- `Microsoft/BVTs_bvt042#1` -- repeated nested imports, modes, and bounded
  `xsl:apply-imports` precedence; and
- `Microsoft/BVTs_bvt043#1` -- principal and imported named-template
  precedence across an included import group.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,395 | 2,398 | +3 |
| Executed successfully | 2,348 | 2,351 | +3 |
| Initialization failures | 775 | 772 | -3 |
| Execution failures | 47 | 47 | 0 |
| Exact expected-result matches | 2,188 | 2,191 | +3 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,191 / 3,173 = 69.05%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52. The `FXST1027` initialization frontier is empty.

## Boundaries

- Resource loading remains sealed, host-bounded, and memory-resident.
- Repeated references are compiled as repeated import branches; no global or
  cross-snapshot module cache is introduced.
- Sibling import order has no execution-order meaning, but it retains the
  standard's import-precedence meaning.
- Mixed sibling include/import graphs and unsupported output properties are
  not inferred by this tranche.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

- The focused nested repeated-import regression passes.
- The unchanged 3,173-case catalog sweep produced the counters above.
- The ordinary workspace verification gate passes.
