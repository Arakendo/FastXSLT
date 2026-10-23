# R0 P08 full selected-profile measurement

Date: 2026-10-23. Tranche 3 of the approved R0 review interval.
Scope: private measurement only; no non-test promotion or feature expansion.

## Run identity

- Processor: **selected legacy reference**, not declaration-driven baseline.
- Preprocessing: **strict XSLT 1.0**, performed by the selected compiler.
- Corpus: unchanged OASIS `XSLT-testsuite-04.ZIP`, 3,173 catalog cases; SHA-256
  `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
- Base commit: `b4c92004dcf6778135d3aff4512c410c83c9d00d`, with pending R0
  implementation/tests. This is a working-tree candidate, not that clean commit.
- Rust/cargo: 1.95.0; Windows host target. Exact toolchain, source/overlay hashes,
  catalog/doubts identities and report hash are in the
  [run manifest](r0-selected-profile-run-identity-2026-10-23.json).
- [Captured full report](r0-selected-profile-full-report-2026-10-23.txt) preserves
  counters, diagnostic partitions, exclusions, mismatches and category totals.

```powershell
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10
```

The script validates the archive and denominator, rejects the incoherent
selected/modern combination and restores both environment selections afterward.
The default remains baseline/modern. A preprocessing-only strict run is still
not a selected-processor run.

## Conserved terminal outcomes

| Outcome | Cases |
| --- | ---: |
| Initialization failure, partitioned in full report | 2,785 |
| Execution failure | 10 |
| Missing catalog input resources before initialization | 3 |
| Exact output comparison | 358 |
| Output mismatch | 2 |
| Comparator limitation | 1 |
| Host parser policy exclusion | 4 |
| Serialization layout policy exclusion | 3 |
| Unusable archival output reference | 5 |
| Unusable archival expected-error expectation after successful execution | 2 |
| **Total** | **3,173** |

The harness asserts this terminal total and separately conserves the expected-
error subset. Initializations and successful executions are intermediate counts
(385 and 375), not additional terminal outcomes. No panic was observed.

Exact comparisons are **358/3,173 (11.28%) for this narrow private profile**.
This does not replace or subtract from the earlier production 2,504/3,173
(78.92%) baseline or baseline/strict 2,501/3,173 measurement. It measures the
frozen admitted slice, not the capability of the complete baseline compiler.
The unsupported majority is a valid measurement result, not authorization to
implement those families during R0.

### Expected-error accounting

Of 431 expected-error catalog cases:

- 125 produced semantic Invalid outcomes (115 initialization, 10 execution);
- 301 failed non-semantically and remain **uncredited**;
- 3 were unavailable before initialization;
- 2 executed but have unusable archival error expectations.

These sum to 431. The report retains the existing `expected-error-credit` label
for the 125 observations, but they are separate from exact output comparisons
and are not a proof of exact standard-code/error-phase conformance for all cases.
The selected route explicitly excludes unsupported, containment and harness
failures from credit. The older baseline reporting rule was not silently
changed in this tranche; comparisons must disclose this accounting difference.

### Mismatches and limitations

- `Lotus/whitespace_whitespace21#1`: retained strict-preprocessing whitespace
  disagreement; original expectation unchanged.
- `Microsoft/Text__78309#1`: previously observed whitespace difference, retained
  as a mismatch rather than normalized away.
- `Microsoft/Elements__78347#1`: expected output is outside the existing
  document/fragment comparator; no new comparator was added.

The [earlier whitespace observations](../../oasis-xslt10/evidence/documents-and-resources/oasis-xslt10-literal-document-path-binding-2026-10-17.md)
remain historical evidence, not automatic corrections to these expectations.
Promotion review must retain or disposition these observations; this measurement
does not assert that every mismatch is either an engine defect or a bad oracle.

## Routing and containment

The existing corpus adapter still acquires and seals resources. Only test-only
options carry the selected profile into the resource compiler. After bounded
graph loading, a private graph router calls the existing selected include,
import, mixed and leaf compilers. Selected lowering owns strict preprocessing,
so the baseline preprocessing graph is neither run first nor used as fallback.
Source preparation, execution, serialization and comparison remain shared.

The router follows the existing group-measurement shape rules; it does not add
graph semantics. Embedded roots and unintegrated composition shapes receive a
distinct `FXHP0001` harness-limitation diagnostic if encountered. Other selected
guards retain their structured unsupported/invalid diagnostics. The report has
no observed `FXHP0001` cases in this replay, not a promise of full graph support.

The corpus-role control now covers baseline/modern, baseline/strict and
selected/strict; it verifies both selected stamps, output parity, bounded
resource admission and the unchanged ordinary adapter duplicate-role refusal.
All profile plumbing remains `cfg(test)`. No public API, production default,
resource authority or accepted semantic ceiling changed.

## Review outcome requested

Validation on this candidate: full repository verification passed (formatting,
Clippy, 2,291 workspace tests, documentation, Markdown links, unsafe-surface and
pinned conformance checks). The official WASM target built successfully; its
incremental cache reported a hard-link-to-copy fallback, not a build failure.
The final replay reproduced the counts above and includes every initialization
diagnostic partition, including the three cases omitted by the usual top-30 list.

All three approved tranches have completed. P08's **before-migration full private
measurement** is now evidenced; its after-migration replay remains open. Whole
obligation count remains 0/9: P01 has not moved the selected slice into a non-test
entry, and downstream closure must be demonstrated on that candidate.

Recommendation for maintainer review: **Continue**, without widening R0, toward
P01 migration and P02/P04/P05/P06 validation of that entry, then P07 controlled
cost/control checks and P08 post-migration replay/P09 final verification.
Select the next bounded review interval before proceeding. Do not promote on
the strength of this measurement alone or chase its unsupported numerator.
