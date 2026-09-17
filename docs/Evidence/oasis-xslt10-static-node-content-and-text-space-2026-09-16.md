# OASIS XSLT 1.0 Static Node Content and Text Space -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing static comment and processing-instruction constructors admit
context-free `xsl:value-of` content, and can `xsl:text` accept its standard
`xml:space` attribute, without admitting a second dynamic constructor engine or
weakening explicit unsupported boundaries?

## Implemented slice

Yes, within two narrow compile-time boundaries.

`xsl:text` now accepts `xml:space="default"` and `xml:space="preserve"` in
addition to its existing `disable-output-escaping` attribute. An invalid
`xml:space` value reports `XTSE0020`; unrelated attributes remain unsupported.
The parsed text node is still the source of the emitted value, so this change
does not introduce a second whitespace-stripping path.

Comment and processing-instruction sequence constructors now fold one
context-free `xsl:value-of` when its expression is already covered by the
bounded literal, literal `concat()`, or literal `substring()` folders. The
instruction validates its attributes and empty content, and
`disable-output-escaping` remains semantically inert in these node-construction
contexts. Source-dependent values, loops, and general dynamic constructor
content remain explicit `FXST1034` or `FXST1036` boundaries.

## Corpus result

Unchanged Lotus `output_output54` now constructs its formula-derived comment
and produces the exact XML-semantic expected result. The two newly initialized
`xml:space` cases advance only to their next honest boundaries: one requires
HTML serialization and one reaches an unsupported ordinary
disable-output-escaping operation. No pass is claimed for either case.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,837 | 1,840 | +3 |
| Initialization failures | 1,298 | 1,295 | -3 |
| Executed successfully | 1,639 | 1,641 | +2 |
| Execution failures | 198 | 199 | +1 |
| XML comparator unsupported | 22 | 23 | +1 |
| Expected-result XML matches | 1,493 | 1,494 | +1 |
| XML comparison mismatches | 116 | 116 | 0 |
| Expected errors unexpectedly succeeding | 6 | 6 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,494 / 2,742 = 54.49%` for standard-operation cases and
`1,494 / 3,173 = 47.08%` for the complete catalog.

## Verification

Focused compiler tests require legal and invalid `xml:space` handling, exact
whitespace preservation, literal `substring()` folding into a comment, and
literal `concat()` folding into a processing instruction. The complete sweep
conserves all 3,173 identities and adds no mismatch, unexpected success, or
panic.
