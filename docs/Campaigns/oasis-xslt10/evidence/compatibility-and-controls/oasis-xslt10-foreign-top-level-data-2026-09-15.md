# OASIS XSLT 1.0 Foreign Top-Level Data -- 2026-09-15

Date: 2026-09-15  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the stylesheet compiler distinguish permitted foreign top-level data from
an invalid unqualified literal top-level element without retaining a new form
of compiled state?

## Implemented slice

Yes. A non-XSLT top-level element with a non-null namespace URI is now ignored
during stylesheet compilation, as required by the XSLT 1.0 stylesheet model.
The compiler does not interpret its attributes as AVTs, execute its children,
retain it in the compiled program, or grant extension authority.

An unqualified non-XSLT top-level element remains invalid and now reports the
`FXST1003` invalid category rather than being classified as an unsupported
FastXSLT capability. XSLT-namespace declarations still pass through the
existing supported/unsupported declaration dispatch.

## Corpus result

Eight unchanged cases move beyond the former blanket `FXST1003` frontier:

- Lotus `namespace_namespace15` and `namespace_namespace16`, plus Microsoft
  `Stylesheet__91802`, become exact expected-result passes;
- Microsoft `Output__78175` executes and becomes a visible whitespace/output
  mismatch;
- Lotus `extend_extend01`, `mdocs_mdocs13`, and `mdocs_mdocs17`, plus Microsoft
  `BVTs_bvt022`, advance to their later extension, `document()`, global-value,
  or literal-result control boundaries.

Microsoft `AVTs__77598` and `Errors_InvalidTopLevelElement` retain explicit
initialization failure because their top-level elements have null namespace
URIs. The first is an invalid AVT placement case and the second directly tests
the non-null-namespace requirement.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,771 | 1,775 | +4 |
| Initialization failures | 1,364 | 1,360 | -4 |
| Executed successfully | 1,587 | 1,591 | +4 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,444 | 1,447 | +3 |
| XML comparison mismatches | 114 | 115 | +1 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,447 / 2,742 = 52.77%` for standard-operation cases and
`1,447 / 3,173 = 45.60%` for the complete catalog.

## Verification

A focused compiler test proves that namespace-qualified foreign data is
ignored even when it contains expression-shaped attribute text, while an
unqualified top-level element remains invalid. The unchanged corpus sweep
conserves all 3,173 identities and exposes every later boundary without adding
an execution failure or panic.
