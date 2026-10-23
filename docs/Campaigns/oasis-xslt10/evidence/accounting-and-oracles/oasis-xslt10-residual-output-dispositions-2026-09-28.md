# OASIS XSLT 1.0 Residual Output Dispositions

- Date: 2026-09-28
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

After the bounded HTML comparator and XML-whitespace correction, 19 successful
executions remained unequal to their archival reference results. Each identity
was reviewed rather than hidden behind broader normalization.

## Historical serialization layout

Eleven cases have the intended result names, values, and HTML structure but
select one historical serializer's optional whitespace, processing-instruction
layout, void-element layout, or Content-Type `meta` placement:

- Lotus `output_output36`;
- Microsoft `BVTs_bvt064`;
- Microsoft `Output__84260`, `Output__84264`, `Output__84271`,
  `Output__84273`, `Output__84277`, `Output__84280`, `Output__84282`, and
  `Output__84285`; and
- Microsoft `Output__84309`.

XSLT 1.0 does not define one byte-exact indentation algorithm for HTML output,
and its HTML Content-Type `meta` insertion rule does not define the archive's
exact surrounding layout. These identities therefore receive exact
`serialization-layout-policy-excluded` dispositions. The comparator remains
strict for authored text and no pass credit is awarded.

## Lossy archival reference bytes

Six cases preserve Unicode source/result characters in FastXSLT while their
archival single-byte reference files replace or transliterate those characters:

- Microsoft `Output__84374` and `Output__84429`; and
- Microsoft `Output__84452`, `Output__84453`, `Output__84454`, and
  `Output__84460`.

The affected references use literal question marks or ASCII substitutions for
characters including Greek, Japanese, Cyrillic, Tamil, and Latin Extended-A.
They cannot distinguish correct Unicode serialization from historical lossy
conversion. The six identities receive exact
`unusable-reference-result-excluded` dispositions and no pass credit.

## Host collation policy

Microsoft `Sorting__77977` combines multilingual text with an archival
single-byte reference that loses the original characters. Its requested text
sort also does not identify a portable collation. Reproducing the archived
order would therefore select both a host collation and a lossy conversion
policy. The identity receives the narrower existing
`host-collation-policy-excluded` disposition; FastXSLT does not claim its
current deterministic ordering as universal XSLT 1.0 collation behavior.

## Remaining visible results

The classifications initially reduce the visible mismatch frontier from 19
cases to one: `Microsoft/Keys_PerfRepro2#1`. The subsequent document-child
wildcard-pattern correction moves that case to its next explicit unsupported
execution boundary rather than crediting its materially different result. The
broad `Microsoft/BVTs_bvt067#1` HTML stress result remains the sole
comparator-unsupported case.

The subsequent HTML serialization/reference-boundary tranche makes that result
comparable and assigns it an exact uncredited archival-reference disposition.
The current ledger has no visible mismatch and no comparator-unsupported case;
see
[OASIS XSLT 1.0 HTML Serialization Reference Boundaries](oasis-xslt10-html-serialization-reference-boundaries-2026-09-29.md).

## Measurement effect

The strict exact lower bound remains 2,261 / 3,173 (71.26%). Initialization and
execution remain 2,440 and 2,392 respectively. Visible mismatches fall from 19
to one, serialization-layout-policy exclusions rise from 6 to 17, unusable
reference-result exclusions rise from 62 to 68, and host-collation-policy
exclusions rise from 4 to 5. This classification tranche changes no engine
behavior, corpus byte, or pass credit; the later pattern correction and its
lifecycle movement are recorded independently.

## Verification

Each disposition is an exact bounded identity with negative controls. The full
measurement continues to conserve all 3,173 catalog cases.

```powershell
cargo test -p fastxslt --all-features oasis_serialization_layout_policy_exclusion_is_exact_and_bounded
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
cargo test -p fastxslt --all-features oasis_host_collation_policy_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
