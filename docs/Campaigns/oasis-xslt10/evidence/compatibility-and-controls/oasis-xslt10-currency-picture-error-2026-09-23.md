# OASIS XSLT 1.0 Currency-Picture Error

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Does the XSLT 1.0 compatibility path reject U+00A4 CURRENCY SIGN in a
`format-number()` picture, including when the picture is selected from source
data at invocation time?

## Normative boundary

[XSLT 1.0 section 12.3](https://www.w3.org/TR/xslt-10/#function-format-number)
uses the JDK 1.1 `DecimalFormat` pattern language and explicitly states that a
format pattern must not contain U+00A4. Currency-sign support was added after
the JDK version selected by XSLT 1.0.

FastXSLT now validates the resolved XSLT 1.0 picture before formatting and
reports structured `XTDE1310 / invalid`. The check applies to literal,
variable, and path-selected pictures and does not change the modern
`format-number()` path.

## Corpus effect

The unchanged 3,173-case OASIS sweep changes as follows:

- `Microsoft/XSLTFunctions__currency#1` moves from an unexpected successful
  execution to its required observed execution error;
- expected-error unexpected successes fall from six to five;
- expected execution errors observed at execution rise from 32 to 33;
- `Lotus/numberformat_numberformat07#1` no longer receives exact-result credit
  for an implementation-specific recovery that emitted the forbidden currency
  sign as a literal prefix;
- successful executions fall from 2,175 to 2,173, execution failures rise from
  58 to 60, and the strict exact-result lower bound moves from 2,026 to 2,025 / 3,173
  (63.82%).

The one-pass decrease is intentional. A cataloged recovery output does not
override the Recommendation's invalid-picture rule.

## Remaining expected-error audit

The five remaining unexpected successes do not form one engine defect:

- `Microsoft/Namespace__77665#1`, `Microsoft/Namespace__77675#1`, and
  `Microsoft/Output__78176#1` have explicit suite doubt annotations saying the
  Recommendation does not require an error;
- `Microsoft/Errors_err031#1` constructs `xml:space`; the XSLT static namespace
  context includes the implicit `xml` binding and neither XSLT 1.0 nor
  Namespaces in XML forbids an element with that expanded name;
- `Microsoft/Miscellaneous__84001#1` contains empty CDATA sections, which are
  valid XML and contribute no stylesheet text node.

FastXSLT therefore keeps those successes visible and uncredited rather than
manufacturing non-standard failures.

## Verification

- focused formatter tests cover literal and dynamically resolved pictures,
  including a quoted currency sign;
- a focused transform test proves a source-selected currency picture becomes
  `XTDE1310`;
- the unchanged Microsoft corpus case produces `XTDE1310`;
- the complete local sweep conserves all 3,173 identities.

The archive remains local and unmodified. This is compatibility evidence, not
a broad conformance claim.

