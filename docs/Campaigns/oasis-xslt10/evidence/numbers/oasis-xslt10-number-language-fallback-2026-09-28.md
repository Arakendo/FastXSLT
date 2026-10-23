# OASIS XSLT 1.0 Number-Language Compatibility Slice

Date: 2026-09-28  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Four unchanged Microsoft cases were stopped at FastXSLT's explicit
`xsl:number/@lang` boundary even though their selected formats did not require
a new locale-dependent numbering implementation. Three use decimal numbering;
the fourth requests the ordinary Latin alphabetic sequence under a Nordic
language hint.

## Change

The XSLT 1.0 compatibility compiler now admits a static `lang` hint when its
effect is already determined by the selected format:

- any language is inert when every selected format token is decimal; and
- `da`, `en`, `fi`, `no`, and `sv` may accompany the existing Latin upper- or
  lower-case alphabetic sequences.

The hint does not select the host locale, load locale data, or mutate the
shared formatter. A non-literal hint, a dynamic format combined with `lang`,
or any other locale-sensitive token/language combination remains explicitly
unsupported as `FXST1052`. Unsupported numbering tokens continue to use the
standard decimal fallback rather than pretending the requested sequence is
implemented.

## Corpus result

Three unchanged cases become exact expected-result matches:

- `Microsoft/Number__84719#1`
- `Microsoft/Number__84723#1`
- `Microsoft/Number__84724#1`

`Microsoft/Number__84722#1` also initializes and executes with the expected
semantic number values, but its UTF-16LE reference contains malformed line
endings. After its BOM, the first expected line break is encoded as
`0D 0A 00 0D 0A 00` rather than `0D 00 0A 00`. Decoding therefore produces
non-whitespace U+0A0D and U+0D00 characters. The artifact is classified as an
exact `unusable-reference-result` and receives no pass credit.

After implementation and classification:

- exact expected-result matches reach `2,224 / 3,173 = 70.09%`;
- initialized cases reach 2,435;
- successful executions reach 2,387;
- initialization failures fall to 735;
- comparator gaps rise to 54 because an additional case reaches comparison;
- visible XML mismatches fall to five, all doubt-annotated; and
- unusable archival reference results rise to 57.

## Boundaries

- This is not general locale-sensitive numbering support.
- No ambient locale, collation, operating-system data, or host authority is
  consulted.
- The malformed UTF-16 reference remains named and uncredited.
- Modern XSLT numbering semantics and the existing typed number formatter are
  unchanged.

## Verification

```powershell
cargo test -p fastxslt xslt10_number_accepts_locale_hints_when_the_selected_sequence_is_invariant
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```

## Standards source

- [W3C XSLT 1.0 section 7.7.1](https://www.w3.org/TR/1999/REC-xslt-19991116#convert)
