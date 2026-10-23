# XSLT30 call-template names and default parameters

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 cases `call-template-1102` and
`call-template-1601` pass through the normal sealed-resource, compilation,
transform-set, and XML-comparison path.

- `call-template-1102` proves that leading underscores remain valid in local
  variable, named-template, and template-parameter names while a constructed
  value crosses the call boundary.
- `call-template-1601` supplies one constructed parameter and evaluates a
  second parameter's string `select` default in the called template.

Neither case required new engine semantics. They extend the conserved corpus
surface over the existing invocation-owned variable and parameter machinery.

## Boundary

This tranche does not broaden parameter identity to expanded QNames, add typed
parameter conversion, or infer support for arbitrary parameter sequence
constructors. `call-template-0702` remains visibly not run at the expanded-QName
binding boundary.

## Accounting

The complete 42-case `insn/call-template` denominator advances from 23 to 25
passes, with one profile exclusion and 16 visible defaults. Across the conserved
XSLT30 denominators this produces 496 passes, 12 engine-unsupported cases,
2,879 profile exclusions, and 112 visible defaults out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt30_call_template_inventory_tests --all-features`
