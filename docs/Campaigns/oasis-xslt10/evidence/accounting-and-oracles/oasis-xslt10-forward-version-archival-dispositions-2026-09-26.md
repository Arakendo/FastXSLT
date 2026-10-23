# OASIS XSLT 1.0 Forward-Version Archival Dispositions

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Do three successfully executing forward-version cases expose FastXSLT defects,
or do their archival expectations require separate compatibility dispositions?

## Findings

### `Microsoft/Namespace__78214`

The stylesheet declares `version="2.0"` and selects seven nodes with
`xsl:value-of`. FastXSLT's modern core applies the modern sequence rule and
emits all seven string values with the effective separator. The archival
expected result contains only the first value because the suite exercised an
XSLT 1.0 processor applying forward-compatible behavior.

Changing the shared modern evaluator to match that expectation would regress
the selected XSLT 2.0/3.0 semantics. The case is therefore classified as
`legacy-processor-profile-excluded`: it remains in the conserved denominator,
receives no pass credit, and records future pressure for an explicitly selected
legacy processor profile rather than an alternate semantic engine.

### `Lotus/ver_ver05` and `Lotus/ver_ver06`

The unchanged stylesheets declare forward versions `17.1` and `17.2` and their
selected literal output text is respectively `17.1` and `17.2`. FastXSLT emits
those values. The immutable expected files instead contain `1.1` and `1.2`,
contradicting the stylesheets themselves.

The two cases are therefore classified as
`unusable-reference-result-excluded`. They remain in the denominator and
receive no pass credit. The upstream artifacts are not rewritten.

## Measurement

The complete unchanged 3,173-case catalog was rerun after adding exact,
case-identity-bounded classifications.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,342 | 2,342 | 0 |
| Executed successfully | 2,291 | 2,291 | 0 |
| Exact XML-semantic matches | 2,125 | 2,125 | 0 |
| XML comparison mismatches | 69 | 66 | -3 |
| Unusable reference-result exclusions | 4 | 6 | +2 |
| Legacy processor-profile exclusions | 0 | 1 | +1 |

The strict exact lower bound remains 2,125 / 3,173 (66.97%). This tranche
improves disposition accuracy only; it adds no feature and awards no pass.

## Verification

- Exact bounded unit tests admit only the three named case identities.
- The modern multi-node `xsl:value-of` regression suite remains unchanged.
- The complete catalog reports 66 visible mismatches, six unusable reference
  results, one legacy processor-profile exclusion, and zero panics.
