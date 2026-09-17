# OASIS XSLT 1.0 Comment Delimiter Recovery -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT implement XSLT 1.0's permitted recovery for constructed comment
content containing `--` or ending in `-` without weakening the modern
stylesheet boundary?

## Implemented slice

Yes. When the effective stylesheet compatibility context is XSLT 1.0, the
static comment constructor inserts one space after a hyphen that is immediately
followed by another hyphen and after a final hyphen. The recovered value is
therefore legal XML comment content and remains owned by the existing compiled
comment instruction.

The decision is made during compilation. Later stylesheet versions retain the
existing explicit `FXST1037` boundary, and the change adds no runtime version
branch, parser recovery, serializer workaround, or second comment
representation.

## Corpus result

All ten cases previously stopped at `FXST1037` now produce their exact
XML-semantic expected results. This includes the two Lotus discretionary
add-space cases and eight Microsoft delimiter/entity combinations.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,840 | 1,850 | +10 |
| Initialization failures | 1,295 | 1,285 | -10 |
| Executed successfully | 1,641 | 1,651 | +10 |
| Execution failures | 199 | 199 | 0 |
| Expected-result XML matches | 1,494 | 1,504 | +10 |
| XML comparison mismatches | 116 | 116 | 0 |
| Expected errors unexpectedly succeeding | 6 | 6 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,504 / 2,742 = 54.85%` for standard-operation cases and
`1,504 / 3,173 = 47.40%` for the complete catalog.

## Verification

A focused compiler test covers consecutive and final hyphens under an XSLT 1.0
root and requires the same content to remain unsupported under an XSLT 3.0
root. The complete sweep conserves all 3,173 identities and adds no mismatch,
unexpected success, or panic.
