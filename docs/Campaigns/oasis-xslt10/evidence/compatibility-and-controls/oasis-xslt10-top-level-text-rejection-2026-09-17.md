# OASIS XSLT 1.0 Top-Level Text Rejection -- 2026-09-17

Date: 2026-09-17  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The stylesheet compiler formed its top-level list with the shared
`meaningful_children` helper, but the declaration loop silently continued when
a retained child had no expanded element name. Consequently, non-whitespace
text at stylesheet top level was discarded even though it makes the stylesheet
invalid.

## Repair

Compilation now validates every meaningful top-level child before declaration
processing. A retained non-element child reports invalid `FXST0008` at that
child's source location. Whitespace-only text, XML comments, and processing
instructions remain excluded by the existing stylesheet-tree rules; foreign
namespace top-level elements retain their separately reviewed behavior.

## Corpus result

Unchanged Microsoft
`Stylesheet_InvalidStylesheetMustThrowException` now reports its required
initialization error instead of succeeding. The exact-result lower bound stays
at 1,512 because this is an expected-error correction.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,860 | 1,859 | -1 |
| Initialization failures | 1,275 | 1,276 | +1 |
| Executed successfully | 1,661 | 1,660 | -1 |
| Expected errors observed during initialization | 401 | 402 | +1 |
| Expected errors unexpectedly succeeding | 6 | 5 | -1 |
| Expected-result XML matches | 1,512 | 1,512 | 0 |
| XML comparison mismatches | 118 | 118 | 0 |
| Execution panics | 0 | 0 | 0 |

The five remaining unexpected successes stay named in the conserved report and
require independent include, namespace, output, or legacy-corpus analysis.

## Verification

A focused compiler regression requires non-whitespace top-level text to fail as
invalid while the existing invalid-versus-unsupported diagnostic distinction
remains intact. The complete sweep conserves all 3,173 identities.
