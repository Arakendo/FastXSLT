# XSLT30 call-template focus and recursion

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 cases `call-template-0601` and
`call-template-0701` now pass through the normal sealed-resource,
compilation, transform-set, and XML-comparison path.

- `call-template-0601` proves that named invocation does not replace the
  caller's current source node or focus. The same declaration also carries a
  match pattern and participates independently in ordinary template dispatch.
- `call-template-0701` recursively composes named and matched templates,
  inherited atomic parameters, sequence-constructor defaults, and child
  selection while preserving the active source focus.

Both cases also reuse the modern invocation-owned local content-variable path
admitted by `call-template-0501`.

## Boundary

The adjacent `call-template-0702` case remains visibly not run. Its parameter
declarations, variable references, and supplied arguments use different
lexical prefixes for the same expanded QName. FastXSLT currently rejects that
broader parameter-identity shape explicitly at compilation. This tranche does
not replace runtime binding keys or infer a public QName parameter contract.

## Accounting

The complete 42-case `insn/call-template` denominator advances from 21 to 23
passes, with one profile exclusion and 18 visible defaults. Across the
conserved XSLT30 denominators this produces 494 passes, 12 engine-unsupported
cases, 2,879 profile exclusions, and 114 visible defaults out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt30_call_template_inventory_tests --all-features`

