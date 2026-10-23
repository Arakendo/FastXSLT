# OASIS XSLT 1.0 Conflict-Resolution Reference Dispositions

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Four Microsoft conflict-resolution cases successfully execute but do not match
their catalog-selected expected XML:

- `ConflictResolution__77781`, `77782`, and `77783`; and
- `ConflictResolution__77879`.

The first three correctly select the last of two equal-precedence,
equal-priority rules, including explicit `0`, `-0`, and `1` controls. Their
only semantic difference is a whitespace-only source suffix selected by a
final `xsl:apply-templates`: FastXSLT preserves it because the stylesheet does
not declare `xsl:strip-space`, while the archival expected result assumes the
host parser removed it before transformation.

Case `77879` supplies an explicit priority of `0.25` on the union pattern
`book|book/title`. Both branches therefore outrank the later default-priority
exact-name rules. FastXSLT selects the union rule for both `book` and
`book/title`, matching the catalog purpose that both alternatives take the
explicit higher priority. The catalog-selected `77879_output.txt` instead
selects the lower-priority red `book` rule. A second archival file,
`77879.txt`, selects the higher-priority brown rule, further demonstrating that
the selected reference is not a stable semantic oracle.

## Disposition

Cases `77781` through `77783` receive exact `host-parser-policy-excluded`
dispositions. FastXSLT does not gain ambient source-whitespace stripping merely
because a historical host parser supplied it.

Case `77879` receives an `unusable-reference-result-excluded` disposition. The
engine retains standard explicit-priority selection and does not reproduce the
catalog-selected file's lower-priority rule choice.

All four cases remain named in the conserved 3,173-case denominator and receive
no pass credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,125 exact XML comparisons out of 3,173 (66.97%). Visible XML
mismatches fall from 22 to 18. Host-parser-policy exclusions rise from 17 to
20, and unusable-reference-result exclusions rise from 37 to 38.

## Verification

The measurement adapter's exact bounded-list tests cover all four identities.
The existing template-conflict tests continue to verify equal-rank use-last
selection and explicit priority ordering independently of corpus disposition.

```powershell
cargo test -p fastxslt --all-features oasis_host_parser_whitespace_policy_exclusion_is_exact_and_bounded
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
