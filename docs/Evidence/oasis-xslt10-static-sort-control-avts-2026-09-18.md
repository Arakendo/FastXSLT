# OASIS XSLT 1.0 Static Sort-Control AVTs

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:sort` admit context-free literal AVTs for `data-type` and `order`
without adding dynamic sort-policy evaluation to the runtime?

## Result

Yes. The compiler now folds the exact one-expression AVT whose expression is
an XPath string literal. Values such as `data-type="{'number'}"` and
`order="{'descending'}"` become the same typed `SortDataType` and `SortOrder`
plans as their literal attribute spellings.

Variable-valued, path-valued, mixed-text, or otherwise dynamic sort controls
remain `FXST1044 / unsupported`. The runtime has no new AVT evaluator or
version branch, and sort selection, focus, stability, work charging, and
cancellation continue through the existing path.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- `Microsoft/BVTs_bvt005#1` and `Microsoft/Output__84025#1` leave their first
  `FXST1044` boundary and expose a later dynamic computed-name boundary.
- `Lotus/sort_sort33#1`, whose sort order is genuinely variable-valued, moves
  from an inaccurate `XTDE0030 / invalid` classification into the explicit
  `FXST1044 / unsupported` frontier.
- The net `FXST1044` frontier falls from 17 to 16 cases. The later
  `FXST1047` frontier rises from 21 to 23 because the two static-AVT cases now
  reach it.
- Aggregate lifecycle and comparison counts are unchanged: 1,940 cases
  initialize, 1,844 execute successfully, and 1,565 compare exactly.

This is bounded language admission plus honest frontier refinement, not an
exact-result gain.

## Verification

- A focused compiler test admits literal `number` and `descending` AVTs and
  keeps a variable-valued control unsupported.
- The complete local OASIS measurement conserves all identities and exposes
  the three case transitions above.
- The ordinary workspace verification gates pass.
