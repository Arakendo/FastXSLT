# OASIS XSLT 1.0 Extension-Element Boundary

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT prevent a declared extension instruction from being compiled as
an ordinary literal result element while extension execution and fallback
semantics remain unsupported?

## Changes

- Literal-element compilation now checks the stylesheet root's validated
  `extension-element-prefixes` declaration.
- Elements in a declared prefixed or `#default` extension namespace report
  explicit unsupported `FXST1059` before result construction.
- An unused valid extension declaration remains admissible; declaring a
  namespace does not itself execute an extension or grant authority.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Expected errors observed during initialization | 405 | 407 | +2 |
| Expected-error cases with unexpected success | 7 | 5 | -2 |
| Initialized | 1,474 | 1,472 | -2 |
| Executed successfully | 1,311 | 1,309 | -2 |
| Expected-result XML matches | 1,202 | 1,202 | 0 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 163 | 0 |

The unchanged `Microsoft/Errors_err034#1` and
`Microsoft/Stylesheet__91812#1` cases no longer execute declared extension
instructions as literal result elements. Their former executions were not
compatibility passes, so the exact-result lower bound remains 1,202.

## Boundaries

This is an explicit unsupported boundary, not extension support. FastXSLT does
not invoke foreign code, acquire new authority, implement `xsl:fallback`, or
select extension availability/error timing. Those require deliberate review
against the host-neutral and explicit-authority architecture.

## Verification

- A focused compiler test proves prefixed and `#default` extension elements are
  rejected while an unused declaration remains admissible.
- The complete local OASIS measurement proves both false successes disappear
  and no exact result regresses.
- No upstream corpus byte was edited.
