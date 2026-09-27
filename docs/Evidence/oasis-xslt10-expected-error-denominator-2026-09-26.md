# OASIS XSLT 1.0 Expected-Error Denominator

- Date: 2026-09-26
- Status: Verified conserved measurement tranche
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The OASIS sweep already distinguished failures observed during initialization
and execution, but it did not conserve those observations against the catalog's
complete expected-error denominator. A large body of correct rejection behavior
therefore remained visible only as phase counters and could not be credited or
audited as one closed family.

## Conserved denominator

The pinned catalog contains 431 `execution-error` cases. Every identity now has
exactly one of these dispositions:

- 390 observe a structured failure during initialization;
- 33 observe a structured failure during execution;
- five execute successfully but have unusable archival error expectations; and
- three cannot enter the engine because the archive omits two principal sources
  and one supplemental stylesheet.

The measurement asserts the conservation equation on every run:

```text
431 catalog expected-error cases
  = 423 observed-error credits
  +   5 unusable-error-expectation exclusions
  +   3 infrastructure exclusions
```

No expected-error case succeeds without a named disposition.

## Reporting boundary

Observed errors are a separate 423 / 431 (98.14%) behavioral denominator. They
do not become expected-result comparison passes and do not erase the eight
excluded identities. For orientation only, exact expected-result matches plus
observed expected errors account for 2,557 / 3,173 catalog cases (80.59%); this
combined figure is not a conformance claim because unsupported standard cases,
profile selection, comparator coverage, and archival exclusions remain open.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
