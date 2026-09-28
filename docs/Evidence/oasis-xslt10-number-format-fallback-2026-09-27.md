# OASIS XSLT 1.0 Number-Format Fallback -- 2026-09-27

Date: 2026-09-27  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

FastXSLT rejected an `xsl:number` format token unless it was one of the
engine's specialized decimal, Latin alphabetic, Roman, or Greek alphabetic
forms. That was too strict for XSLT 1.0. Section 7.7.1 requires a processor that
does not implement the numbering sequence named by a token to use format token
`1`; it also supplies token `1` when the format contains no alphanumeric format
token.

The missing behavior appeared after UTF-16 stylesheet admission exposed several
Microsoft international-numbering cases. It did not justify pretending that
FastXSLT implements Arabic, Hebrew, Indic, Thai, Cyrillic, Japanese, Korean, or
Chinese numbering systems.

## Change

The shared static/dynamic number-format parser now:

- preserves the existing specialized decimal, Latin, Roman, and Greek forms;
- maps an otherwise unsupported alphanumeric token to decimal token `1`;
- uses default token `1` for an empty format;
- supplies token `1` for a punctuation-only format while retaining its prefix
  and suffix;
- retains the default period separator when no separator token occurs between
  format tokens; and
- accepts a valid static `letter-value="alphabetic"` when the effective token
  is decimal, while continuing to reject the unsupported reinterpretation of a
  Roman token.

Focused tests cover static compilation, dynamic/shared parsing, unsupported
Unicode token fallback, empty and punctuation-only formats, prefix/suffix
retention, multiple-number formatting, and the empty-number-list boundary.

## Corpus result

Eight unchanged cases now compile, execute, and compare exactly:

- `Microsoft/Number__84699#1`
- `Microsoft/Number__84705#1`
- `Microsoft/Number__84714#1`
- `Microsoft/Number__84715#1`
- `Microsoft/Number__84716#1`
- `Microsoft/Number__84717#1`
- `Microsoft/Number__84725#1`
- `Microsoft/Number__84726#1`

Three additional cases now execute but remain visible mismatches:

- `Microsoft/Number__84692#1` has an archival doubt stating that its reference
  output is missing sub-elements.
- `Microsoft/Number__84700#1` expects optional language-specific Arabic and
  full-width numbering sequences. FastXSLT permissibly falls back to decimal
  rather than claiming those numbering systems.
- `Microsoft/Number__91027#1` expects punctuation-only/prefix punctuation to be
  reused as separators and emits punctuation for empty number lists. FastXSLT
  retains the already conserved empty-list behavior and the XSLT default period
  separator where no between-format-token separator exists.

Four related cases remain at the explicit unsupported `xsl:number @lang`
boundary or the distinct Roman/alphabetic reinterpretation boundary. They are
not admitted by this fallback slice.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,384 | 2,395 | +11 |
| Executed successfully | 2,337 | 2,348 | +11 |
| Initialization failures | 786 | 775 | -11 |
| Execution failures | 47 | 47 | 0 |
| Exact expected-result matches | 2,180 | 2,188 | +8 |
| Visible mismatches | 6 | 9 | +3 |

The conservative all-catalog exact-match ratio is now
`2,188 / 3,173 = 68.96%`. Expected-error credit remains 423 / 431 and comparator
gaps remain 52.

## Boundaries

- Decimal fallback is not evidence of locale-specific numbering support.
- No `lang` behavior, locale acquisition, ambient system-locale dependency, or
  new public configuration was selected.
- The two historical-reference differences and the explicitly doubt-annotated
  empty reference remain visible rather than being converted into pass credit.
- Modern XPath/XSLT numeric typing and the existing specialized token behavior
  are unchanged.

## Verification

- Focused compiler/runtime tests exercise the shared format parser and sequence
  formatter.
- The unchanged 3,173-case catalog sweep produced the counters above.
- The ordinary workspace verification gate passes.
