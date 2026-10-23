# OASIS XSLT 1.0 Legacy HTML Output Versions

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Can the existing legacy HTML serializer admit explicitly selected historical
HTML versions without weakening the modern unsupported-version diagnostic or
creating a second serializer?

## Method

- Inventory all eight cases stopped at runtime `SESU0013`.
- Retain HTML 5/5.0 as the existing specialized mode.
- Route explicit HTML 1/1.0, 4/4.0, and 4.01 selections through the existing
  legacy HTML serializer.
- Keep empty and unknown version values unsupported.
- Preserve the unchanged XSLT30 `output-0194` `SESU0013` regression for version
  `0.0`.
- Rerun the unchanged 3,173-case OASIS catalog.

## Result

Seven of the eight former `SESU0013` cases now execute. Five become exact
XML-semantic comparisons; two reach the already-visible non-document HTML
comparison boundary. The remaining empty-version case retains explicit
`SESU0013` rather than silently selecting a version.

The conserved totals are 2,140 initialized cases, 995 initialization failures,
2,080 successful executions, 60 execution failures, 1,929 exact XML-semantic
matches, and 61 mismatches. The strict compatibility lower bound is
`1,929 / 3,173 = 60.79%`.

## Boundaries

- This admits version selection into the existing general legacy HTML
  serialization behavior; it does not claim every historical browser quirk.
- Empty, zero, and otherwise unrecognized HTML versions remain unsupported.
- The XSLT30 unsupported-version case remains unchanged, demonstrating that
  the broader legacy list is not equivalent to accepting arbitrary versions.
- The two newly exposed comparison-frontier cases remain uncredited.

## Reproduction

```powershell
cargo test -p fastxslt --all-features legacy_html_serialization_accepts_a_general_nested_result_tree
cargo test -p fastxslt --all-features reports_html_serialization_parameter_failures_before_shape_selection
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier SESU0013
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
