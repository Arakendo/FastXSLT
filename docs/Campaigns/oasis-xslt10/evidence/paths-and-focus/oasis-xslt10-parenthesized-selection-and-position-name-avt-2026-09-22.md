# OASIS XSLT 1.0 Parenthesized Selection and Position Name AVT

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Can FastXSLT admit redundant outer parentheses around XSLT 1.0 location paths
and unions, and can a computed-attribute name use `position()` while retaining
the edition's last-wins duplicate-attribute recovery?

## Method

- Strip only balanced parentheses that enclose the complete XSLT 1.0 location
  path before invoking the existing typed path parser. Keep the modern XPath
  parser unchanged.
- Apply the same bounded normalization to XSLT 1.0 `xsl:apply-templates` and
  `xsl:for-each` selection parsing so a parenthesized union reuses the existing
  typed union representation.
- Add a typed focus-position part to the private XSLT 1.0 computed-attribute
  name AVT representation. Resolve it from the current sequence focus and
  charge the existing XPath-operation work domain.
- Carry XSLT 1.0 duplicate recovery as a compile-selected fact on computed
  attributes. When a dynamic name collides with a literal or earlier computed
  attribute, replace the earlier expanded name before child construction.
- Retain static XTDE0410 validation and runtime duplicate rejection for modern
  stylesheets.
- Exercise the focused parser and runtime contracts, then rerun the unchanged
  3,173-case OASIS catalog.

## Result

The unchanged Microsoft `BVTs_bvt002` case now initializes, executes, and
compares XML-semantically. Its parenthesized descendant path and path union use
the typed selection machinery, its `attr{position()}` names produce `attr1`
through `attr20`, and its duplicate dynamic attributes retain the last value.

Together with one other case enabled by the parenthesized XSLT 1.0 path
normalization, the tranche adds two exact matches. Conserved totals are 2,152
initialized cases, 983 initialization failures, 2,096 successful executions,
56 execution failures, 1,944 exact XML-semantic matches, and 62 mismatches. The
strict compatibility lower bound is `1,944 / 3,173 = 61.27%`.

## Boundaries

- This is bounded XSLT 1.0 compatibility syntax, not general XPath grouping.
  The modern location-path parser continues to reject the parenthesized form.
- The name AVT admits `position()` as a typed part; it does not add a general
  dynamic XPath evaluator to computed names.
- Duplicate recovery is selected during compilation and applies only to XSLT
  1.0 computed attributes. Modern duplicate construction remains an error.
- Attribute identity uses expanded names, not lexical prefixes.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_outer_parentheses_reuse_the_typed_location_path
cargo test -p fastxslt --all-features xslt10_focus_position_composes_one_computed_attribute_name
cargo test -p fastxslt --all-features xslt10_duplicate_result_attributes_recover_with_the_last_value
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/BVTs_bvt002#1
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
