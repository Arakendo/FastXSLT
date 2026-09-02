# XSLT30 `mode-1902` Private Initial-Mode Admission

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Native test set `tests/attr/mode/_mode-test-set.xml`.
- Unchanged case `mode-1902`, principal stylesheet `mode-1902.xsl`, imported
  stylesheet `mode-1101.xsl`, environment `mode-11`, initial mode `X`, and
  expected dynamic error `XTDE0045`.

## Method

Both stylesheet modules and the external source are read into a bounded sealed
snapshot. The ordinary resource compiler resolves and compiles the import. The
case adapter verifies that the imported stylesheet's supported principal-output
properties (`method="xml"`, `encoding="UTF-8"`, and `indent="no"`) are inherited
when the principal does not shadow them, and that the principal's private mode
declaration is retained with its source location.

The unchanged initial-mode request is then submitted through normal
transform-set admission. Admission must reject mode `X` before source parsing
or template execution and report the native error code, request identity, and
principal declaration location.

## Results

- `mode-1902` reports `XTDE0045` during request admission.
- The structured failure retains request identity `mode-1902` and the
  `mode-1902.xsl` declaration location.
- A focused compiler test proves that a private named mode is retained while
  broader public component visibility remains explicitly unsupported.
- The complete 169-case mode denominator now records 87 passes, 48 profile
  exclusions, and 34 visible default not-run cases.
- Conserved XSLT30 accounting becomes 410 passed comparisons, 3
  engine-unsupported cases, 54 profile exclusions, and 100 visible defaults
  across 567 cases.

## Bounded import-output prerequisite

The unchanged imported stylesheet carries ordinary output settings. Previously,
single-import compilation rejected every unshadowed imported output property,
preventing the visibility semantics from being reached. This slice admits
inheritance only for the already-supported `method`, `encoding`, and `indent`
properties from one imported program. Principal properties still win. Other
unshadowed properties and multiple-import output composition retain the prior
explicit unsupported boundary.

## Limitations

This evidence admits only denial of a named private mode as an externally
selected initial mode. It does not admit public/final component composition,
packages, abstract modes, overriding rules, imported conflicting visibility
recovery (`mode-1905`), or a public visibility-inspection API. Private mode
identity and location remain compiled internal semantics used by invocation
admission.
