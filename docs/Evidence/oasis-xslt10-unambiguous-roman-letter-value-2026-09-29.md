# OASIS XSLT 1.0 Unambiguous Roman `letter-value` -- 2026-09-29

Date: 2026-09-29  
Status: Verified bounded semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

FastXSLT already implements the fixed `i` and `I` numbering tokens as lower- and
upper-case Roman numerals, but compilation rejected those tokens when the
stylesheet supplied `letter-value="alphabetic"`.

XSLT 1.0 defines `i` and `I` as fixed Roman sequences. `letter-value` resolves
ambiguity for letter-based numbering sequences; it does not replace the fixed
meaning of an otherwise unambiguous token.

## Repair

The static validator now admits either valid `letter-value` lexical value over
the existing Roman token styles. This adds no numbering algorithm, locale data,
dynamic attribute-value-template support, or host policy. Unknown lexical
values remain invalid and unimplemented numbering sequences remain unsupported.

Focused compiler and runtime tests prove that the admitted alphabetic spelling
does not reinterpret `i` or `I`: both retain their existing Roman output.

## Corpus result

The unchanged `Microsoft/Number__84720#1` case now initializes, executes, and
matches exactly. The complete conserved sweep records:

- 3,173 catalog cases;
- 2,441 initialized cases;
- 2,392 successful executions;
- 2,261 exact matches (71.26%);
- 729 initialization failures;
- 49 execution failures;
- zero visible mismatches and zero comparator gaps; and
- 423 / 431 expected-error credits (98.14%).

## Boundaries

- This slice covers only static `letter-value` applied to already implemented,
  unambiguous numbering tokens.
- It does not admit dynamic `format`, `lang`, or `letter-value`, locale-sensitive
  numbering sequences, or additional compound-format behavior.
- This is compatibility evidence from a locally acquired archive, not a formal
  XSLT 1.0 conformance claim.

## Verification

```powershell
cargo test -p fastxslt number_admits_static_letter_values_when_existing_tokens_are_unambiguous
cargo test -p fastxslt xslt10_number_applies_latin_greek_and_roman_format_tokens
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Number__84720#1'
./scripts/verify.ps1
```

## Standards source

- [XSLT 1.0, 7.7.1 Number to String Conversion Attributes](https://www.w3.org/TR/1999/REC-xslt-19991116#convert)
