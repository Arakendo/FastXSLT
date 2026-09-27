# XSLT30 call-template required-parameter validation

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 negative case `call-template-2101` now reports
`XTSE0690` during stylesheet compilation.

The existing post-link validator already resolved every `xsl:call-template`
against the final named-template table. It now also compares each call's
supplied arguments with the selected declaration and rejects an omitted
required non-tunnel parameter. This is performed after module composition, so
the check uses the declaration that execution would actually select.

## Boundary

The static obligation applies to required non-tunnel parameters. A required
tunnel parameter may be supplied through the dynamic tunnel chain and is not
rejected merely because it is absent from the immediate call. Type checking of
supplied arguments, expanded-QName parameter identity, and broader sequence-type
validation remain separate work.

## Accounting

The complete 42-case `insn/call-template` denominator advances from 28 to 29
passes, with one profile exclusion and 12 visible defaults. Across the conserved
XSLT30 denominators this produces 500 passes, 12 engine-unsupported cases,
2,879 profile exclusions, and 108 visible defaults out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt30_call_template_inventory_tests --all-features`
- focused compiler regression for an omitted required non-tunnel parameter
