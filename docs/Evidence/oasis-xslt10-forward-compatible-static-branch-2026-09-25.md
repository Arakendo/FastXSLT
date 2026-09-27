# OASIS XSLT 1.0 Forward-Compatible Static Branch

Date: 2026-09-25  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a stylesheet whose declared version is newer than FastXSLT's supported
XSLT 3.0 level use `system-property('xsl:version')` to keep an unknown future
instruction in a branch that is provably not selected?

## Change

- Static XSLT introspection now reports `1` inside the explicit XSLT 1.0
  compatibility profile and `3.0` outside that profile.
- Boolean tests may reuse the existing namespace-aware static introspection
  folder in modern and forward-compatible stylesheets.
- For a stylesheet version greater than `3.0`, an `xsl:choose` branch whose
  introspection test folds to `false` retains an empty body instead of eagerly
  compiling an instruction that cannot be instantiated.
- Ordinary supported-version branches remain fully validated. A selected
  unknown instruction still fails explicitly with `FXST1006`.

The change does not add a generic optimizer, suppress errors in XSLT 1.0 or
3.0 stylesheets, or implement an unknown future instruction.

## Corpus result

Two unchanged Xalan cases, `ver05` and `ver06`, now initialize and execute.
They leave the `FXXP1002` initialization frontier and select their supported
fallback branches. Their archival expected files say `1.1` and `1.2`, while
the unchanged stylesheets and actual results say `17.1` and `17.2`, so both
remain visible comparison mismatches and receive no exact-pass credit.

`ver01` reaches its fallback branch but remains explicitly unsupported because
basic `xsl:message` is currently admitted only by the XSLT 1.0 compatibility
compiler. `ver07` correctly observes FastXSLT's `3.0` processor level, selects
its XSLT 2.0 branch, and remains explicitly unsupported at
`xsl:result-document`. Neither boundary was weakened to improve the count.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,326 | 2,328 | +2 |
| Executed successfully | 2,278 | 2,280 | +2 |
| Initialization failures | 809 | 807 | -2 |
| XML comparison mismatches | 82 | 84 | +2 |
| Exact XML-semantic matches | 2,111 | 2,111 | 0 |

The strict lower bound therefore remains 2,111 / 3,173 (66.53%). The increased
mismatch count records newly observed behavior rather than a regression in a
previously executing case.

Subsequent corpus-accounting review classified `ver05` and `ver06` as named
unusable-reference-result exclusions because their immutable expected text
contradicts their stylesheets. They remain uncredited. See
[OASIS XSLT 1.0 Forward-Version Archival Dispositions](oasis-xslt10-forward-version-archival-dispositions-2026-09-26.md).

## Verification

- A compiler test proves an unselected future-version instruction is deferred.
- A compiler counter-test proves a selected unknown instruction in a supported
  `version="3.0"` stylesheet remains unsupported.
- An end-to-end transform proves the fallback branch executes through the
  ordinary engine.
- The unchanged, hash-verified 3,173-case catalog was rerun.
