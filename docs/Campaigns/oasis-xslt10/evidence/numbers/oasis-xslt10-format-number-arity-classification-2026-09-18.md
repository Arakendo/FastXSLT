# OASIS XSLT 1.0 `format-number()` Arity Classification

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT distinguish a statically invalid `format-number()` call from a
valid-arity call whose operands or picture remain outside the admitted decimal
formatting slice?

## Result

Yes. The private formatting parser now counts top-level function arguments
while respecting nested parentheses and quoted strings. A structurally complete
call with an arity other than two or three reports `XPST0017 / invalid` before
operand and picture capability selection. Calls with a valid arity but an
unimplemented operand, picture, or decimal-format-name form continue to report
`FXXP1009 / unsupported`.

Malformed or unbalanced syntax is not guessed to be an arity error. It remains
on the existing explicit syntax or unsupported boundary.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- Four unchanged cases leave `unsupported/FXXP1009` and report
  `invalid/XPST0017`:
  - `Microsoft/FormatNumber_extraParameters#1`;
  - `Microsoft/XSLTFunctions__extraParameter#1`;
  - `Microsoft/XSLTFunctions__without2rdParameter#1`;
  - `Microsoft/XSLTFunctions__withoutAnyParameter#1`.
- The `FXXP1009` initialization frontier falls from 14 to 10 cases. Those ten
  calls have a valid arity and retain their honest implementation boundary.
- All aggregate lifecycle and comparison counts are unchanged: 1,940 cases
  initialize, 1,844 execute successfully, and 1,565 compare exactly.

This is diagnostic and frontier refinement, not additional language admission
or a conformance-pass increase.

## Verification

- Parser tests distinguish zero-, one-, and four-argument calls from a
  valid-arity unsupported operand shape.
- Stylesheet compilation tests require `XPST0017 / invalid` for each wrong
  arity.
- The complete local OASIS measurement conserves all catalog identities and
  exposes the four exact case transitions above.
- The ordinary workspace verification gates pass.
