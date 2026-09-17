# OASIS XSLT 1.0 Malformed AVT Classification

Date: 2026-09-17  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT distinguish malformed attribute-value-template delimiters from a
well-formed AVT expression that is merely outside the current evaluator?

## Change

The literal-attribute compiler now validates AVT delimiter structure before it
selects a supported expression representation. It:

- recognizes doubled `{{` and `}}` as literal braces outside expressions;
- ignores braces inside quoted XPath string literals;
- rejects unmatched opening or closing braces;
- rejects nested unquoted opening braces; and
- rejects empty or whitespace-only expressions.

Malformed forms report `XTSE0370 / invalid` with the existing stylesheet source
location. Well-formed expressions outside the private evaluator continue to
report `FXST1031 / unsupported`.

## Corpus result

Eight unchanged cases move from `FXST1031 / unsupported` to
`XTSE0370 / invalid`: Microsoft AVT cases `77588`, `77589`, `77592`, `77595`,
and `77596`, plus error cases `err012`, `err013`, and `err044`.

The conserved 3,173-case census is otherwise unchanged:

- initialized: 1,939;
- executed successfully: 1,844;
- exact XML-semantic matches: 1,564;
- initialization failures: 1,196; and
- execution failures: 95.

This is diagnostic correction, not additional conformance credit.

## Verification

- Focused compiler tests cover empty, unmatched, nested, escaped, and
  quote-contained brace forms.
- The local corpus trace identifies exactly eight `XTSE0370` cases.
- No upstream corpus byte or expected result was changed.

