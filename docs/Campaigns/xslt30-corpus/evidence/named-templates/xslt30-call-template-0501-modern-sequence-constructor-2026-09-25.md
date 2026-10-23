# XSLT30 `call-template-0501` modern sequence constructor

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 case `call-template-0501` now passes through the
normal sealed-resource, compilation, transform-set, and XML-comparison path.

Its source-matched template constructs a local temporary tree by executing
`xsl:value-of select="a"`, passes that value to a named template, and lets a
second optional parameter use its sequence-constructor default. The result is
the unchanged `<out>test,pvar2 default data</out>` assertion.

## Implementation boundary

The complete invocation-owned `SequenceTreeVariable` path is no longer
restricted to XSLT 1.0 stylesheets or to a single modern `apply-templates`
shape. Any untyped local content variable may use the ordinary compiled
instruction sequence and then materialize its result as an invocation-owned
temporary document.

The compact static constructor remains an optimization and the complete path
remains safe and work-charged. Variables declaring `as` still require a
separately admitted typed-value path; this change does not select general
sequence types, cross-invocation temporary-tree sharing, or a public tree
representation.

## Accounting

The complete 42-case `insn/call-template` denominator advances from 20 to 21
passes, with one profile exclusion and 20 visible defaults. Across the
conserved XSLT30 denominators this produces 492 passes, 12 engine-unsupported
cases, 2,879 profile exclusions, and 116 visible defaults out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt20_content_variable_executes_value_of_before_named_template_binding --all-features`
- `cargo test -p fastxslt xslt30_call_template_inventory_tests --all-features`

