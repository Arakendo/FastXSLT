# XSLT30 call-template modern numeric flow

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 case `call-template-2001` now passes. It carries a
constructed source value and numeric values through three nested named
templates, converts one parameter with `number($pvar2)`, and evaluates
`444 + $t1num` after another parameter transfer.

Modern `number($variable)` and binary arithmetic variable operands use the
modern zero-or-one selection rule. The existing XSLT 1.0 compatibility plans
remain separate and retain first-in-document-order conversion. This prevents
the corpus admission from weakening modern cardinality behavior merely because
the selected case supplies singleton values.

## Conserved behavior

- content-built local variables remain invocation-owned temporary trees;
- passing a variable preserves its typed runtime value rather than flattening
  every argument at compilation;
- numeric conversion and arithmetic remain charged XPath work;
- nested named calls do not leak call-local parameter values;
- the unchanged inline XML assertion is compared by the corpus adapter.

## Accounting

The complete 42-case `insn/call-template` denominator advances from 29 to 30
passes, with one profile exclusion and 11 visible defaults. Across conserved
XSLT30 denominators this produces 501 passes, 12 engine-unsupported cases,
2,879 profile exclusions, and 107 visible defaults out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt30_call_template_inventory_tests --all-features`
- full workspace verification gate

