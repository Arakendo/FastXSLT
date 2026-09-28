# OASIS XSLT 1.0 Forward-Compatible Attributes -- 2026-09-27

Date: 2026-09-27  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Two unchanged Microsoft `ForwardComp` cases stopped during compilation:

- an unknown unqualified attribute on `xsl:template`; and
- the future lexical value `indent="sure"` on optional `xsl:output/@indent`.

Both stylesheets declare version `1.5`. XSLT 1.0 section 2.5 requires an
attribute not allowed by XSLT 1.0, or an optional attribute value not allowed
by XSLT 1.0, to be ignored in forwards-compatible processing mode.

## Change

The shared XSLT-element attribute validator now ignores otherwise unsupported
attributes in the bounded XSLT 1.x forward-compatible interval. Existing
validation of `xml:space`, supported attributes, and strict supported-version
behavior remains unchanged.

The output compiler also treats an invalid future lexical for its optional
boolean properties as absent in that same interval. The ordinary XSLT 1.0 and
modern-version parsers remain strict, so this does not broaden accepted values
for supported versions or add a runtime branch.

A focused regression combines both behaviors and retains an XSLT 1.0 negative
sentinel for the unknown attribute.

## Corpus result

The unchanged cases `Microsoft/ForwardComp__91847#1` and
`Microsoft/ForwardComp__91848#1` now compile, execute, and compare exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,408 | 2,410 | +2 |
| Executed successfully | 2,359 | 2,361 | +2 |
| Initialization failures | 762 | 760 | -2 |
| Execution failures | 49 | 49 | 0 |
| Exact expected-result matches | 2,199 | 2,201 | +2 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,201 / 3,173 = 69.37%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52.

## Boundaries

- Recovery is selected statically only for the bounded XSLT 1.x
  forward-compatible interval.
- Required attributes and values are not made optional by this tranche.
- `xml:space` validation remains strict.
- Supported XSLT `1.0`, `2.0`, and `3.0` declarations retain their existing
  attribute validation.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

- The focused positive and strict-negative compiler regression passes.
- Both unchanged corpus cases compare exactly.
- The complete 3,173-case catalog sweep produced the counters above.

## Normative reference

- [XSLT 1.0 section 2.5, Forwards-Compatible Processing](https://www.w3.org/TR/xslt-10/#forwards)
