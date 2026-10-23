# OASIS XSLT 1.0 Inferred HTML Comparison

Date: 2026-09-28  
Status: Verified local-harness evidence; not an engine or public comparator contract  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

FastXSLT already applies the XSLT 1.0 output-method inference rule: when no
method is selected and the first significant result element is an unnamespaced
`html` element, serialization uses HTML behavior. The local OASIS comparator,
however, applied its bounded HTML normalization only when `xsl:output`
explicitly selected `method="html"`.

Five successfully executed inferred-HTML cases therefore remained classified
as unparseable rather than as a pass or visible mismatch.

## Change

After bounded XML-declaration and DOCTYPE handling, the test-only comparator
now recognizes an ASCII-case-insensitive unnamespaced lexical `html` root when
no explicit output method exists. It applies the same bounded HTML lexical
normalizer already used for explicit HTML output.

This does not change output-method selection, engine serialization, semantic
result construction, or the upstream expected bytes. Names such as `htmlish`
do not activate the rule, and an explicit non-HTML method remains non-HTML.

## Conserved result

The five former comparator gaps become:

| Disposition | Cases |
| --- | ---: |
| Exact semantic comparison pass | 3 |
| Visible comparison mismatch | 2 |
| Total | 5 |

The exact cases are:

- `Lotus/output_output05#1`;
- `Lotus/select_select74#1`; and
- `Microsoft/BVTs_bvt047#1`.

`Microsoft/BVTs_bvt064#1` and `Microsoft/BVTs_bvt098#1` remain uncredited as
visible mismatches.

Combined with the independently recorded result-namespace fixup, the strict
lower bound reaches **2,254 / 3,173 (71.04%)**. Comparator gaps fall to 12 and
visible mismatches are 17. Initialization remains 2,440, successful execution
remains 2,392, and expected-error credit remains 423 / 431.

## Boundaries

- This is not an HTML parser, DOM, or HTML conformance claim.
- It does not make inferred HTML serialization byte-exact.
- It does not turn a parsed inequality into a pass.
- It is compiled only for the local test/workbench measurement configuration.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_html_comparator
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
