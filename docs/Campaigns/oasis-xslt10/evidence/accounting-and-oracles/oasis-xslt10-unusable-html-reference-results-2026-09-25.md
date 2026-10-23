# OASIS XSLT 1.0 unusable HTML reference results

Date: 2026-09-25

## Result

Three unchanged Microsoft HTML-output cases in the archival OASIS suite point
to expected-result files whose complete content is only the three-byte UTF-8
byte-order mark `EF BB BF`:

- `Output_EmptyElement1`;
- `Output_MethodEqualsHtmlWithoutIndentSet`;
- `Output_UseLiteralResultElementHead`.

Each stylesheet necessarily constructs a non-empty HTML result, and two case
descriptions explicitly require observable HTML serialization behavior. The
zero-content reference files therefore cannot decide semantic or serialization
equivalence. The local measurement now gives these exact identities the
explicit `unusable-reference-result-excluded` disposition instead of reporting
three misleading engine mismatches.

This is a harness classification only. It does not alter the immutable suite,
engine execution, HTML serialization, or the pass numerator. Empty semantic
results remain comparable for cases with usable reference evidence.

## Corpus effect

The complete archival measurement remains at **2,116 exact XML-semantic
matches out of 3,173 catalog cases (66.69%)**. Initialized cases remain 2,328,
successful executions remain 2,281, comparator-unsupported outcomes remain 77,
and visible XML mismatches fall from 72 to 69. Eight historical host-parser
policy exclusions and these three unusable-reference exclusions remain
separately named rather than disappearing from the denominator.

## Validation

- `cargo test -p fastxslt oasis_unusable_reference_result_exclusion_is_exact_and_bounded --all-features`
- `scripts/measure-oasis-xslt10.ps1`
- byte-level inspection of the three pinned reference files
