# OASIS XSLT 1.0 Source Node-Kind Copy-Of Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Archive SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Input baseline | 395 definite unchanged XML passes |
| Result | 403 definite unchanged XML passes |
| Disposition | Source attribute, comment, and processing-instruction copying implemented |

## Outcome

The shared source deep-copy operation now preserves all source node kinds needed
by the admitted node-path `xsl:copy-of` slice. Attributes become pending result
attributes so the containing result element enforces attribute ordering and
duplicate-name rules. Comments and processing instructions retain their data
and PI target through the existing validated constructors. All three kinds use
the normal result-node budget and cancellation path.

The complete local OASIS sweep moved as follows:

| Observation | Prior tranche | This tranche | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 635 | 635 | 0 |
| Execution succeeded | 450 | 461 | +11 |
| Execution failed | 185 | 174 | -11 |
| Definite unchanged XML passes | 395 | 403 | +8 |
| XML comparison mismatches | 29 | 32 | +3 |
| Comparator unsupported | 15 | 15 | 0 |

All 13 prior `FXRT1002` node-kind failures moved forward. Eleven execute to a
comparable result; eight pass and three expose comparison differences. Two
reach a later structured semantic failure, reflected by one additional
`FXRT1007` and one `XTDE0410` observation. No case is credited merely for
passing the former boundary.

The strict standard-operation lower bound is now **403 / 2,742 = 14.70%**;
the complete-catalog lower bound is **403 / 3,173 = 12.70%**. These remain
compatibility measurements, not conformance claims.

Unchanged `Lotus/copy_copy10#1`, previously the representative `FXRT1002`
failure, now copies an attribute-bearing element containing text, a descendant,
and a comment and passes the owned XML comparison. A focused production test
also copies a selected source attribute, comment, and processing instruction
into one result element and checks the serialized kinds and values.

## Architectural conservation

- Source node kinds are translated to the same result-node variants used by
  ordinary construction and temporary-tree execution.
- Attributes remain subject to the result element's existing placement and
  duplicate-name checks; copying does not bypass construction invariants.
- Comments and processing instructions use existing validated constructors.
- Every copied node is charged in the result-node work domain and observes the
  same invocation cancellation policy.
- No XSLT-version branch, legacy result tree, alternate serializer, or public
  representation was introduced.

## Remaining copy-of boundary

The admitted operation still covers node-selecting location paths rather than
general XPath sequences. Atomic values, variables, unions, computed sequences,
validation/type controls, and broader namespace-copy behavior remain explicit.
The remaining 29 compile-time `FXXP1003` cases therefore require expression and
typed-sequence design, not another source-node special case.

The 87 HTML-shape failures, 12 unsupported legacy encodings, and six unsupported
HTML versions exposed by the prior location-path tranche remain serializer
frontiers. They must not be repaired inside `copy-of`.
