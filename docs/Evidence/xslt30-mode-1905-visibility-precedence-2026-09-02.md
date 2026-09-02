# XSLT30 `mode-1905` Visibility Precedence

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Native test set `tests/attr/mode/_mode-test-set.xml`.
- Unchanged case `mode-1905`, principal stylesheet `mode-1904a.xsl`, imported
  stylesheet `mode-1904.xsl`, environment `mode-11`, initial mode `X`, and
  assertion `exists(/scout)`.

## Semantic pressure

The imported module contains two same-precedence declarations for mode `X`
whose visibility values conflict (`final` and `private`). The principal module
declares the same mode `public` at higher import precedence. The lower conflict
therefore does not determine the effective visibility property; rejecting the
imported module in isolation would report a static error that the complete
stylesheet package does not have.

## Method

The bounded single-import compiler identifies an explicit unprefixed principal
visibility declaration. In the imported module it defers only declarations for
the same mode whose unqualified attributes are exactly `name` and `visibility`.
Other declarations and all other mode properties retain normal validation.
The principal public declaration then supplies the effective invocation
visibility, while imported templates remain in the precedence-ranked template
set.

The unchanged source and both stylesheet modules are admitted into one sealed
snapshot. Initial-mode execution selects the principal document rule for mode
`X`; the serialized result is compared as XML with `<scout/>` and satisfies the
suite's native assertion.

## Results

- Unchanged `mode-1905` passes.
- Direct `mode-1904` still reports `XTSE0545`, proving the lower conflict is
  deferred only when the higher-precedence declaration is present.
- Private initial-mode denial from `mode-1902` remains `XTDE0045`.
- The complete mode denominator records 88 passes, 48 profile exclusions, and
  33 visible default not-run cases.
- Conserved XSLT30 accounting becomes 411 passed comparisons, 3
  engine-unsupported cases, 54 profile exclusions, and 99 visible defaults
  across 567 cases.

## Limitations

This is not general package component composition. It admits one unprefixed
mode name and lower declarations containing only the visibility property.
Prefixed/expanded mode identity across this override seam, `final` enforcement,
abstract modes, package exposure, mixed-property lower declarations, multiple
imports, and public inspection remain outside the evidence. The narrow
defer-and-override rule exists to preserve property-level import precedence
without suppressing unrelated validation.
