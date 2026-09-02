# XSLT30 Strip-Space Denominator and Exact Strip-All Execution

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Native test set
  `tests/decl/strip-space/_strip-space-test-set.xml`.
- Unchanged case `strip-space-012`, stylesheet `strip-space-012.xsl`, inline
  environment `strip-space-07`, and native `<assert-xml>` result.
- Accepted [ADR-0012](../ADR/ADR-0012-invocation-owned-whitespace-visibility-view.md)
  exact strip-all policy.

## Method

The suite adapter inventories every native `test-case` before selection and
requires 30 distinct names. A typed first-party overlay gives the denominator a
visible `harness-unsupported/not-run` default and overrides only
`strip-space-012` as selected and passed. The private per-case ledger must agree
with that pass.

For execution, the adapter reads the unchanged stylesheet and inline source,
admits both into a bounded sealed resource snapshot, compiles the stylesheet,
and executes one principal-source request through the normal transform-set
path. The result is compared with the suite's unchanged XML assertion.

## Results

- Complete conserved denominator: 30 cases.
- Selected and passed: 1 (`strip-space-012`).
- Visible default not run: 29.
- Engine unsupported: 0.
- Profile excluded: 0.
- Focused adapter tests: 2 passed.

The selected case proves that `xsl:strip-space elements="*"` removes
whitespace-only source text nodes while preserving the non-whitespace text
selected by ordinary template application. Execution uses the immutable,
invocation-owned visibility view selected by ADR-0012; it does not mutate the
prepared source document or establish a stylesheet-specific prepared tree.

Adding this denominator changes conserved XSLT30 accounting from 531 to 561
cases: 407 passed comparisons, 3 engine-unsupported cases, 54 profile
exclusions, and 97 visible default not-run cases. The lower raw pass fraction is
an intentional consequence of exposing a new complete denominator rather than
hiding its unimplemented siblings.

## Limitations

This evidence admits only exact `elements="*"`. It does not infer element-name
tests, namespace-sensitive name tests, `xsl:preserve-space`, declaration
precedence, schema-aware whitespace behavior, `xml:space` interaction,
temporary-tree stripping, or initial whitespace-node semantics. Those 29 cases
remain visible defaults until their own semantic and assertion dependencies are
understood.
