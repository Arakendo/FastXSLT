# Corpus and Conformance Inputs

The first-party seed cases live in [`../../corpus/`](../../corpus/). This
directory owns policy and future records for external conformance sources,
comparison processors, selection manifests, and result reports.

Before admitting an external suite, record:

1. upstream owner and canonical source;
2. standards edition and suite version or immutable revision;
3. license and redistribution decision;
4. reproducible acquisition and integrity verification;
5. immutable upstream content versus local manifests/adapters;
6. test selection, exclusions, unsupported classification, and harness errors;
7. expected update and audit procedure.

Do not edit upstream expected results to make FastXSLT pass. Keep local
classification or harness corrections in reviewable overlays with rationale.

## Admitted upstream suites

- [W3C QT3 and XSLT 3.0 suite provenance](w3c-test-suites.md) records the two
  Git submodules, immutable revisions, licensing boundary, acquisition check,
  and update procedure.
