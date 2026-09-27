# OASIS XSLT 1.0 unusable error expectations

Date: 2026-09-26

## Result

The complete archival measurement previously reported five successful
executions as `expected-error-unexpected-success`. Review of the immutable
stylesheets and the suite's own `doubts.xml` shows that none establishes a
required XSLT 1.0 error:

- `Namespace__77665`, `Namespace__77675`, and `Output__78176` are annotated by
  the suite with “The Rec does not say that this has to be an error.”
- `Errors_err031` constructs the valid expanded name `xml:space`; the implicit
  `xml` binding supplies the reserved XML namespace as required.
- `Miscellaneous__84001` contains empty CDATA sections, which contribute no
  characters and do not make the XML stylesheet or its sequence constructors
  invalid.

The measurement now gives these exact identities the explicit
`unusable-error-expectation-excluded` disposition. This is a first-party
overlay classification only: the upstream archive remains immutable,
FastXSLT's behavior is unchanged, and no case receives pass credit.

## Corpus effect

The five cases remain in the conserved 3,173-case denominator. The prior
`expected-error-unexpected-success` count falls from five to zero, while five
named unusable-error-expectation exclusions become visible. All execution,
comparison, failure, and exact-match totals remain unchanged:

- 2,342 initialized;
- 2,291 executed successfully;
- 828 initialization failures;
- 51 execution failures;
- 2,125 exact XML-semantic matches (66.97%);
- 69 visible XML mismatches;
- zero panics.

This classification is not evidence that arbitrary successful execution of an
expected-error case is acceptable. Any future such result remains a visible
failure until its exact archival expectation is independently reviewed.

## Validation

- `cargo test -p fastxslt oasis_unusable_error_expectation_exclusion_is_exact_and_bounded --all-features`
- `scripts/measure-oasis-xslt10.ps1`
- byte-preserving inspection of the five pinned stylesheets
- inspection of the pinned `doubts.xml` metadata

