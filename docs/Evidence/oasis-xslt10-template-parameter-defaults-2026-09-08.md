# OASIS XSLT 1.0 template parameter defaults -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can matched and named templates evaluate source-node defaults and references to
preceding parameters without losing the caller's focus or the value's runtime
kind?

## Changes

- Template parameter defaults may compile a typed location path or an exact
  reference to a preceding parameter in addition to the existing literal
  defaults.
- Runtime binds parameters sequentially. A later default copies an earlier
  atomic value, atomic sequence, source-node sequence, or temporary tree rather
  than converting it through a string-shaped compatibility shortcut.
- Source-path defaults use the existing controlled path evaluator at the
  matched or named-call context node. Atomic and temporary-tree execution do
  not manufacture a principal-source context.
- Template parameter names seed local-binding validation, so a body-local
  variable cannot silently rebind a parameter.
- Compiled retained-capacity accounting includes the new path and variable-name
  storage.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,512 | 1,517 | +5 |
| Executed successfully | 1,347 | 1,351 | +4 |
| Expected-result XML matches | 1,235 | 1,239 | +4 |
| XML comparison mismatches | 86 | 86 | 0 |
| Execution failures | 165 | 166 | +1 |

The exact-result gains are unchanged Lotus `variable49`, `select09`, and
`position94` plus Microsoft `bvt096`. `position94` supplies both parameters at
every call; compiling its otherwise-unused bare-name defaults is therefore
necessary, but those defaults do not affect the result. The initialization
gain also advances Microsoft `78402`, an expected-error case with a forward
parameter reference, to the correct runtime unbound-variable failure
`FXRT0002`. Its now-visible execution failure accounts for the one-counter
increase and is not a standard-case regression. Microsoft `78161` remains a
correct initialization failure because its body attempts to rebind a template
parameter.

The strict standard-operation lower bound becomes
`1,239 / 2,742 = 45.19%`; the conservative all-catalog ratio becomes
`1,239 / 3,173 = 39.05%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit general XPath expressions in parameter defaults,
forward parameter dependencies, dynamic errors hidden by an override, source
paths while executing against atomic or temporary-tree focus, or cross-frame
sharing. It does not add a second compatibility runtime or change modern XPath
cardinality semantics. No corpus bytes or expected results were changed.

## Verification

- A focused runtime test exercises source-node and dependent defaults through
  both matched and named template calls.
- A focused compile test preserves the duplicate parameter/local-binding
  error.
- The unchanged Lotus and Microsoft cases above were traced against their
  upstream expected output or expected-error disposition.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
