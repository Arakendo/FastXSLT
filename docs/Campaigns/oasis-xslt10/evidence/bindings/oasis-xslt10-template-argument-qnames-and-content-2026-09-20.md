# OASIS XSLT 1.0 Template-Argument QNames and Content

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can template parameters and supplied arguments compare by expanded QName, and
can `xsl:apply-templates` carry an XSLT 1.0 content-built argument through the
same bounded argument machinery already used by `xsl:call-template`?

## Implemented slice

Matched- and named-template parameter declarations and `xsl:with-param` names
now normalize lexical QNames to the compiler's canonical expanded-name key.
Prefix aliases for the same namespace therefore bind one parameter without
runtime namespace lookup.

`xsl:apply-templates`, `xsl:apply-imports`, and `xsl:next-match` now compile an
argument without `select` through the existing bounded content constructor.
The admitted forms remain literal text, literal constructed content, and the
already bounded XSLT 1.0 constructor plans. A `select` and content remain
mutually exclusive. Source-node selections continue through the existing
charged path evaluator.

This reuses the ordinary invocation-owned parameter frame. It does not create a
legacy parameter store, add runtime namespace authority, or broaden the
content-constructor grammar beyond its existing limits.

## Corpus result

Four unchanged cases become exact XML-semantic matches:

- `Lotus/variable_variable45#1`
- `Microsoft/Variables__84026#1`
- `Microsoft/Variables__84034#1`
- `Microsoft/Variables__84036#1`

The existing missing-name error in `Microsoft/Variables__84035#1` remains an
initialization-time invalid-input outcome.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,073 | 2,077 | +4 |
| Initialization failures | 1,062 | 1,058 | -4 |
| Executed successfully | 1,971 | 1,975 | +4 |
| Exact XML-semantic matches | 1,842 | 1,846 | +4 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound becomes `1,846 / 3,173 = 58.18%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- A focused regression supplies content-built QName arguments through both
  `xsl:call-template` and `xsl:apply-templates`, using different declaration
  and reference prefixes for the same namespace.
- The complete conserved sweep moved exactly four cases from initialization
  failure to exact output without changing mismatch or execution-failure
  totals.
- No upstream corpus byte was edited.
