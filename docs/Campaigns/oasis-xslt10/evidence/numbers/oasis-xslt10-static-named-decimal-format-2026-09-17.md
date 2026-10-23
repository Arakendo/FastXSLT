# OASIS XSLT 1.0 Static Named Decimal Format -- 2026-09-17

Date: 2026-09-17  
Status: Verified bounded semantic slice and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

After unnamed decimal-format support, 28 cases still stopped at named
`xsl:decimal-format` declarations. A substantial subset uses a literal QName as
the third `format-number()` argument. Both the declaration name and the call
site QName can therefore be resolved in their static namespace contexts and
linked during compilation.

## Implementation

The compiler now resolves named decimal-format declaration QNames, composes
same-name non-conflicting declarations, and resolves literal third-argument
QNames at each expression site. The selected immutable format is copied into
the compiled expression. Runtime evaluation therefore uses the same direct
formatting plan as the unnamed case and performs no QName lookup or stylesheet
policy branching.

An undeclared requested format stops explicitly at `FXST1092`. This is
particularly important for include/import cases: general decimal-format
composition across separately compiled modules remains outside this tranche,
so a module-local lookup cannot silently fall back to the standard format.
Variable and computed third-argument names remain unsupported expression
shapes. Digit-family substitution is covered by the subsequent
[digit-family tranche](oasis-xslt10-decimal-digit-family-2026-09-17.md).

## Corpus result

Relative to the unnamed-format baseline:

- initialized cases rise from 1,923 to 1,934;
- executed-successfully cases rise from 1,715 to 1,724;
- exact expected-result matches rise from 1,549 to 1,558;
- initialization failures fall from 1,212 to 1,201;
- execution failures rise from 208 to 210 as later boundaries become visible;
- two expected-error cases move from initialization observation to execution
  observation, leaving the conserved total unchanged; and
- visible XML mismatches and comparator-unsupported outcomes remain 137 and
  23 respectively.

All nine newly executed ordinary cases compare exactly. The expected-error
ledger contains 401 initialization observations, 23 execution observations,
and four doubt-annotated unexpected successes. No corpus panic occurs.

## Verification

Focused tests cover unnamed and named static selection, missing-name
classification, namespace-aware QName resolution through the common compiler,
duplicate-property conflict, and non-distinct symbols. The complete local
measurement conserves all 3,173 identities: 1,934 initialize, 1,724 execute
successfully, 1,558 compare exactly, 137 remain visible mismatches, and 23
reach comparator-unsupported outcomes.
