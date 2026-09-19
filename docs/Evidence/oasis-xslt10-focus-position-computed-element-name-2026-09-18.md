# OASIS XSLT 1.0 Focus-Position Computed-Element Name

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a computed-element name that composes static text with `position()` reuse
the existing sequence focus without admitting arbitrary AVT expressions?

## Result

Yes. Under XSLT 1.0 static context, one exact `prefix{position()}suffix` name
form now compiles as a typed focus-position dynamic name. Execution formats the
already-established sequence focus position, composes the lexical name, and
passes it through the same private runtime `QName` validation and namespace
resolution owner as path-valued computed names.

The path and focus-position forms share one private dynamic-name enum and one
result-construction branch. This removed the source-unit pressure produced by
parallel instruction variants and keeps retention accounting and semantic
inspection exhaustive without exposing the representation publicly.

General AVT expression composition, multiple expressions, variable paths, and
dynamic namespace AVTs remain unsupported.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- `Microsoft/Value-of_ValueOf_ConcatTextNodesIntoSingle#1` leaves `FXST1047`,
  initializes, executes, and matches its XML-semantic expected result.
- The `FXST1047` frontier falls from 2 to 1.
- The strict expected-result lower bound rises from 1,566 to 1,567.
- Aggregate lifecycle counts become 1,945 initialized, 1,848 executed
  successfully, 1,190 initialization failures, and 97 execution failures.

## Verification

- A focused runtime test constructs two differently named elements from one
  two-item focus and verifies their serialized result.
- The complete local OASIS measurement conserves every case identity and
  records the exact frontier-to-pass transition.
- The ordinary workspace verification gates pass.
