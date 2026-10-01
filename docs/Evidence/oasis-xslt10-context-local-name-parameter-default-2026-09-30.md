# OASIS XSLT 1.0 context `local-name()` parameter-default evidence

Date: 2026-09-30

## Result

Named and matched templates may now retain the exact XSLT 1.0 parameter
default `local-name(.)` as a typed dynamic default. When no matching argument
is supplied, invocation binding obtains the current source element's local
name, charges the node visit, and stores the result as an invocation-local
string value.

A focused regression calls a named template from an `item` source context and
proves that the omitted argument defaults to `item`.

The unchanged `Lotus/mdocs_mdocs16#1` stylesheet now compiles past this former
`FXST1032` frontier. It next stops at a variable-rooted path over a global
secondary document, which requires the unresolved cross-document node-
ownership model. The complete OASIS numerator therefore remains unchanged.

## Boundaries

- Only exact `local-name(.)`, allowing XPath whitespace, is admitted.
- General dynamic template-parameter default expressions remain unsupported.
- The default currently requires a principal-source node context; temporary
  and secondary-document focus are not approximated.
- No resource resolution, prepared-state retention, or public API changes.

## Measurement

The complete hash-verified local OASIS CD04 sweep remains:

- 3,173 conserved catalog cases;
- 2,466 exact XML comparisons (77.72%);
- 2,662 initialized cases;
- 2,615 successful executions;
- 508 initialization failures;
- 47 execution failures;
- 11 visible XML mismatches;
- 4 comparator gaps;
- 423 / 431 expected-error credits.

The unchanged totals are intentional: this slice removes one compiler blocker
and makes the next architectural frontier explicit without claiming a pass.
