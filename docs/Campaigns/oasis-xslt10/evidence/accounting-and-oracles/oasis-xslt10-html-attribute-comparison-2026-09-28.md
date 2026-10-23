# OASIS XSLT 1.0 HTML Attribute Comparison

Date: 2026-09-28  
Status: Local comparator evidence

## Question

Can raw HTML attribute syntax be normalized into XML-readable comparison input
without treating a lexical spelling difference as an engine semantic change?

## Method

The private OASIS HTML comparator now:

- escapes raw ampersands and less-than signs in attribute values;
- preserves XML predefined and numeric references;
- preserves the four bounded archival HTML named references for subsequent
  decoding; and
- keeps the first case-insensitive HTML attribute occurrence so repeated
  boolean spellings do not create duplicate XML attributes.

Engine serialization and upstream expected bytes remain unchanged.

## Result

Two former gaps compare exactly after normalization:

- `Lotus/output_output37#1`; and
- `Microsoft/Output_AmpersandWithinHtmlAttribute#1`.

The broad `Microsoft/BVTs_bvt067#1` stress case remains the sole comparator
gap; the bounded normalizer still cannot turn its complete result into an
XML-readable comparison tree, so it receives no credit.

The strict lower bound rises from **2,256 / 3,173 (71.10%)** to
**2,258 / 3,173 (71.16%)**. Comparator gaps fall from 3 to 1, while 22 visible
mismatches and all lifecycle, expected-error, policy, and archival-exclusion
counters remain unchanged.

## Boundaries

- This is not engine behavior, a public HTML parser, or a general DOM contract.
- Normalization is used only to feed the existing expanded-name/value
  comparator.
- The remaining stress case stays visibly unsupported rather than receiving a
  guessed disposition.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features oasis_html_comparator
./scripts/measure-oasis-xslt10.ps1
```
