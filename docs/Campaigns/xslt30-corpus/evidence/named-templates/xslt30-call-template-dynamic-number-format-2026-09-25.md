# XSLT30 call-template dynamic number format

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 case `call-template-1401` passes through the normal
sealed-resource, compilation, transform-set, and XML-comparison path.

The case supplies a constructed `format` parameter to a named template and
uses `format="{$format}"` on `xsl:number`. Nested ordered-list items prove that
the active source focus remains intact while the dynamic format changes between
alphabetic upper- and lower-case numbering.

FastXSLT now retains modern single-variable number-format AVTs separately from
the XSLT 1.0 compatibility representation. The runtime converts atomic
sequences and source-node sequences with space separation, obtains a temporary
document's string value without mutation, and charges the applicable XPath/XDM
work before parsing the resulting format.

## Boundary

This tranche admits exactly one unqualified variable reference occupying the
complete `format` AVT. General AVT composition, dynamic grouping attributes,
dynamic `letter-value`, broader formatting tokens, and qualified variable names
remain outside the admitted surface. XSLT 1.0 conversion behavior remains on its
existing compatibility path.

## Accounting

The complete 42-case `insn/call-template` denominator advances from 26 to 27
passes, with one profile exclusion and 14 visible defaults. Across the conserved
XSLT30 denominators this produces 498 passes, 12 engine-unsupported cases,
2,879 profile exclusions, and 110 visible defaults out of 3,498 cases.

## Validation

- `cargo test -p fastxslt xslt30_call_template_inventory_tests --all-features`
- `cargo test -p fastxslt number_admits_static_letter_values_only_when_existing_tokens_are_equivalent --all-features`
