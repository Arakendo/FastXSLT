# OASIS XSLT 1.0 Host-Owned Stylesheet XML Limits -- 2026-09-27

Date: 2026-09-27  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The workbench host supplied explicit XML event and depth limits for source
preparation, but stylesheet graph loading still used a private 1,024-event and
64-level parser envelope. Two valid large OASIS stylesheets therefore failed
before compilation even though the measurement host deliberately admitted a
larger bounded XML envelope.

The defect was not a reason to remove XML bounds or raise a hidden constant. It
showed that the existing host-owned policy had not reached every parser entry
point under the same engine lifecycle.

## Change

The stylesheet dependency loader now receives the host-supplied `ParseLimits`
used by the workbench. The limits apply uniformly to the principal stylesheet
and every sealed include/import dependency. Source preparation continues to use
the same supplied values.

Event and depth exhaustion during stylesheet parsing now project as structured
`FXRS0006 / limit` outcomes rather than malformed XML. Focused tests prove both
dimensions: a low host ceiling rejects the same immutable stylesheet that a
larger ceiling admits and executes. Default narrow test helpers remain bounded.

This does not make XML limits optional, grant ambient resource authority, or
select public production defaults. The host owns environment-dependent values;
FastXSLT owns deterministic enforcement.

## Corpus result

Both cases formerly stopped by the private 1,024-event stylesheet ceiling now
compile, execute, and compare exactly without changing the archival inputs:

- `Microsoft/AttributeSets__91046#1`
- `Microsoft/Import__91164#1`

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,382 | 2,384 | +2 |
| Executed successfully | 2,335 | 2,337 | +2 |
| Initialization failures | 788 | 786 | -2 |
| Execution failures | 47 | 47 | 0 |
| Exact expected-result matches | 2,178 | 2,180 | +2 |
| Visible mismatches | 6 | 6 | 0 |

The conservative all-catalog exact-match ratio is now
`2,180 / 3,173 = 68.70%`. Expected-error credit remains 423 / 431 and comparator
gaps remain 52.

## Verification

- Focused workbench tests prove host-selected stylesheet event and depth limits,
  structured limit diagnostics, successful admission under larger bounds, and
  execution after admission.
- The unchanged 3,173-case catalog sweep produced the counters above.
- The ordinary workspace verification gate passes.
