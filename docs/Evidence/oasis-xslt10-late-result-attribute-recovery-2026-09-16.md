# OASIS XSLT 1.0 Late Result-Attribute Recovery -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT exercise XSLT 1.0's permitted recovery for an attribute
constructed after result children without weakening later-edition diagnostics or
hiding expected-error cases?

## Implemented slice

Yes. When an XSLT 1.0 stylesheet contains `xsl:attribute` after result child
construction has begun, compilation excludes that constructor from the body and
otherwise ignores it. The ignored instruction is not evaluated and cannot
escape as a pending result attribute. This choice is made only in XSLT 1.0
static context; later versions continue to report `XTDE0410`.

The implementation does not change the runtime rule that dynamically produced
attributes must precede children. It selects one recovery behavior for the
statically evident XSLT 1.0 form rather than making late attributes generally
valid.

## Corpus result

Five unchanged standard-operation cases become exact XML-semantic
expected-result passes: Lotus `attribset19`, `attribset35`, and `attribset36`,
plus Microsoft `Attributes__78379` and `Attributes__78381`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,821 | 1,826 | +5 |
| Initialization failures | 1,314 | 1,309 | -5 |
| Executed successfully | 1,631 | 1,636 | +5 |
| Execution failures | 190 | 190 | 0 |
| Expected-result XML matches | 1,486 | 1,491 | +5 |
| XML comparison mismatches | 115 | 115 | 0 |
| Expected errors unexpectedly succeeding | 6 | 6 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,491 / 2,742 = 54.38%` for standard-operation cases and
`1,491 / 3,173 = 46.99%` for the complete catalog.

## Verification

The focused XSLT 1.0 production-lifecycle test now includes and ignores a late
attribute after a constructed child. The existing XSLT 3.0 compiler test still
requires `XTDE0410` for the same structural error. The unchanged complete sweep
conserves all 3,173 identities, leaves expected-error accounting unchanged, and
adds no XML mismatch, execution failure, or panic.
