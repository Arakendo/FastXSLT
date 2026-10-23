# OASIS XSLT 1.0 Atomic and Focus Template-Argument Tranche

Date: 2026-09-06  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can bounded atomic and focus-dependent expressions supplied by
`xsl:with-param/@select` reuse the existing template-argument owner instead of
being rejected as unsupported path expressions?

## Change

The template-invocation compiler now recognizes quoted XPath string literals,
`true()`, `false()`, `position()`, and `last()` before attempting location-path
parsing. It retains them as typed `TemplateArgumentValue` variants. Runtime
argument evaluation converts those variants into invocation-owned atomic
values using the current template-call focus, so no second parameter frame or
compatibility representation was introduced.

The change applies equally to `xsl:apply-templates`, `xsl:call-template`,
`xsl:next-match`, and `xsl:apply-imports` because they share the same argument
compiler. General arithmetic, other function calls, and constructed-content
arguments remain outside this bounded tranche.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,330 | 1,333 | +3 |
| Executed successfully | 1,156 | 1,159 | +3 |
| Expected-result XML matches | 1,044 | 1,047 | +3 |
| XML comparison mismatches | 77 | 77 | 0 |
| Execution failures | 174 | 174 | 0 |

All three newly executed cases agree with their unchanged archival expected
results:

- `Lotus/impincl_impincl28#1`
- `Lotus/impincl_impincl29#1`
- `Microsoft/Template_TestParamValueAfterCallTemplate#1`

The Microsoft case first exposed a shared compiler defect: removing excluded
leading `xsl:param` nodes before stylesheet text-run classification caused
whitespace before a parameter to be joined to the following non-whitespace
text and emitted. The compiler now keeps excluded nodes as text-run boundaries
while still omitting them from the instruction sequence. A focused formatted
template sentinel guards that conservation rule. No upstream corpus byte was
edited.

The strict standard-operation lower bound is now
`1,047 / 2,742 = 38.18%`; the deliberately conservative all-catalog ratio is
`1,047 / 3,173 = 33.00%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Verification

- The focused named-template test covers a default integer and explicit
  integer, string, boolean, `position()`, and `last()` arguments through the
  same runtime.
- The complete local OASIS measurement completed with the counters above.
- The complete local OASIS measurement ends with no new mismatch or execution
  failure.

The corpus increase in this tranche comes from quoted strings and the shared
text-run repair. Boolean and focus arguments are focused-test groundwork only;
they receive no corpus credit in this measurement.
