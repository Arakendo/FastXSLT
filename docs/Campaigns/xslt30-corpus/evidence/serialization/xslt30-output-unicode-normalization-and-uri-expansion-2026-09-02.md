# XSLT30 Unicode normalization and URI expansion -- 2026-09-02

| Field | Value |
| --- | --- |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/output/_output-test-set.xml` |
| Cases | `output-0101`, `output-0101a`, `output-0101b`, `output-0146`, `output-0164`, `output-0167`, `output-0169` |
| Dependency | `unicode-normalization` 0.1.25 |
| Cargo checksum | `5fd4f6878c9cb28d874b009da9e8d183b5abc80117c40bbd187a1fde336be6e8` |
| Result | Seven unchanged cases selected and passed |

## Semantic result

FastXSLT now supports `normalization-form="NFC"` for the private XML, XHTML,
HTML, and text serialization paths. Ordinary text and attribute expansion
applies character maps first, leaves substituted strings untouched, normalizes
the remaining character runs, and then performs method-specific escaping.
CDATA-selected text is normalized before CDATA construction.

URI-valued `href` attributes use the serialization-standard order independently
of the requested normalization form: normalize the complete value to NFC,
percent-encode non-ASCII UTF-8 bytes, then apply ordinary attribute escaping.
Character maps do not rewrite a URI attribute whose URI expansion is enabled.
The HTML result-shape validator admits only the unchanged
`html/body/div/a[@href]` hierarchy exercised by this tranche; this is not a
claim for the complete HTML/XHTML URI-attribute vocabulary.

The native assertions establish:

- composed `Á` output for XHTML, XML, and text methods;
- exact file-backed XML and text serialization for `output-0167` and
  `output-0169`;
- the upstream XHTML serialization fragment for `output-0146`; and
- the exact normalized URI substring
  `%EF%AD%8F/%C3%A5rsrapport/%C3%A5r/2005?x=y` for all four HTML cases.

The earlier `normalization-form="none"` controls remain byte-preserving. At
this tranche, `NFD`, `NFKC`, `NFKD`, and `fully-normalized` remained
unsupported. The subsequent
[US-ASCII CDATA tranche](xslt30-output-us-ascii-cdata-normalization-2026-09-02.md)
admits NFD; compatibility forms and fully-normalized output remain unsupported.

## Dependency review

The runtime dependency is exact-pinned at 0.1.25. Its published manifest
declares `MIT OR Apache-2.0`, Rust 1.36, and one normal dependency,
`tinyvec` with its `alloc` feature. The lockfile resolves that edge to
`tinyvec` 1.12.0 and `tinyvec_macros` 0.1.1; their manifests declare
`Zlib OR Apache-2.0 OR MIT` and `MIT OR Apache-2.0 OR Zlib`, respectively.
Those terms permit FastXSLT's MIT distribution. The normalization tables report
Unicode 17.0.0. The crate exposes iterator-based NFC/NFD/NFKC/NFKD operations
defined against Unicode Standard Annex #15; FastXSLT currently consumes only
NFC.

Source inspection found a narrowly contained dependency-owned unsafe surface
in Hangul decomposition and composition: one unsafe helper and five unsafe
operation sites use `char::from_u32_unchecked` after range arithmetic. Each
site has a local safety explanation, and the crate otherwise denies unsafe
code. The resolved `tinyvec` and `tinyvec_macros` source introduces no unsafe
operation. This remains third-party trusted code under ADR-0003; FastXSLT adds
no first-party unsafe code and does not relax the workspace lint.

This inspection is not a security audit and did not run Miri, fuzzing, or an
external vulnerability scanner against the dependency. Replaceability is
localized to the serializer's private normalization helpers. Reopen admission
if the exact package or transitive versions move, maintenance or security
evidence changes, Unicode-version behavior breaks a standards case, or a
suitable safer implementation can satisfy the same semantic and performance
requirements.

## Accounting and limits

All emitted UTF-8 bytes still pass through `BudgetedString`, so normalization,
percent expansion, character-map output, and escaping remain charged against
the invocation's serialized-byte budget. NFC iteration is streaming for the
ordinary and URI paths. The character-map composition helper buffers only the
current unmapped text run; CDATA construction owns one normalized string before
splitting embedded `]]>` delimiters. Both inputs are already bounded by the
invocation-owned semantic result, but this tranche does not establish a
separate normalization-work unit or allocator-exact peak-memory formula.

With these seven promotions, `decl/output` records 187 passes, one profile
exclusion, and 44 visible default not-run cases. The 531-case conserved XSLT30
subtotal records 385 passes, three engine-unsupported cases, 51 profile
exclusions, and 92 visible default not-run cases.

## Validation

The focused output inventory ran all 87 output-owner tests successfully. The
workspace-wide format, Clippy, test, documentation, and Markdown-link gates are
recorded in the implementing commit's validation.
