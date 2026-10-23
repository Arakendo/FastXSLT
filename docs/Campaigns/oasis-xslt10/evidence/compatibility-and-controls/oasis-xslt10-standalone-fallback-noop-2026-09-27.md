# OASIS XSLT 1.0 Standalone Fallback No-Op -- 2026-09-27

Date: 2026-09-27  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged `Microsoft/Fallback__91817#1` stylesheet places `xsl:fallback`
directly in an XSLT 1.0 template. Its body contains a value-producing
instruction, but the expected result is empty: outside an extension or
forward-compatible instruction, `xsl:fallback` itself has no effect and its
sequence constructor is not instantiated.

FastXSLT previously treated the element as an unsupported instruction during
compilation.

## Change

The sequence compiler now recognizes standalone `xsl:fallback` only under the
XSLT 1.0 compatibility static context, validates its admitted attributes, and
emits no instruction. It deliberately does not compile the fallback body.

Fallback children of unknown extension or forward-compatible instructions
continue through their existing selected-content path. Modern supported
versions remain strict; this tranche does not reinterpret a supported XSLT 2.0
or 3.0 stylesheet as though FastXSLT were an XSLT 1.0-only processor.

## Corpus result

The unchanged `Microsoft/Fallback__91817#1` now compiles, executes, and compares
exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,416 | 2,417 | +1 |
| Executed successfully | 2,364 | 2,365 | +1 |
| Initialization failures | 754 | 753 | -1 |
| Execution failures | 52 | 52 | 0 |
| Exact expected-result matches | 2,204 | 2,205 | +1 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,205 / 3,173 = 69.49%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52.

## Boundaries

- The no-op is compile-selected for the XSLT 1.0 compatibility context.
- The ignored body is not compiled or executed.
- Selected fallback content beneath an unknown instruction is unaffected.
- The remaining version-2.0 fallback corpus cases stay behind AR-0019's open
  processor-profile/version-mode boundary.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

- A focused compiler regression proves that surrounding instructions remain in
  order while invalid content inside the standalone fallback is not compiled.
- The unchanged corpus case compares exactly.
- The complete 3,173-case catalog sweep produced the counters above.

## Normative reference

- [XSLT 1.0 section 15, Fallback](https://www.w3.org/TR/xslt-10/#fallback)
