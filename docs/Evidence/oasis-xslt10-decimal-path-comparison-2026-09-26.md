# OASIS XSLT 1.0 Decimal Path Comparison

- Date: 2026-09-26
- Status: Verified engine and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged `Lotus/output_output60` executed to a plausible HTML result but
omitted the conditional `style="color:red"` attribute for source value `-1.5`.
The private `path < numeric-literal` plan parsed selected source-node values as
integers, so decimal XPath numbers silently made the condition false.

## Implementation

The private boolean plan now retains the finite XPath numeric literal as double
bits and converts each selected node string with the established XPath-number
lexical parser. Comparison remains existential across the selected node set,
charged through the existing path and string-value owners, and selected during
compilation. No recurring stylesheet-version branch enters execution.

A focused engine test covers a negative decimal, zero, and positive decimal and
requires the conditional attribute to appear only for the negative value.

The local corpus comparator also recognizes that only an HTML DOCTYPE's name is
ASCII case-insensitive. The external identifier suffix remains lexically exact;
XML output retains case-sensitive comparison; and different names remain
different. This lets the harness proceed past `html` versus `HTML` without
weakening production serialization or DTD input policy.

## Corpus result

`Lotus/output_output60` now constructs the expected conditional attribute and
otherwise matches the archival semantic content. Its HTML result still is not
an XML document or fragment because the HTML output method emits void elements
according to HTML rules. The case therefore moves from a visible engine
mismatch to the existing named HTML-comparator gap and receives no pass credit.

The conserved sweep is now:

- exact matches: **2,171 / 3,173 (68.42%)**;
- initialized: **2,371**;
- executed successfully: **2,326**;
- execution failures: **45**;
- visible mismatches: **9**, including two doubt-annotated numeric cases;
- comparator gaps: **53**; and
- expected-error credit: **423 / 431**.

## Boundaries

This tranche does not add a semantic HTML comparator, normalize arbitrary
DOCTYPE syntax, accept scientific XPath 1.0 numeric literals, alter modern
XPath typing, or credit a corpus pass for semantically plausible output.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_path_numeric_comparison_accepts_decimal_node_values
cargo test -p fastxslt --all-features oasis_html_comparator_treats_only_the_doctype_name_as_ascii_case_insensitive
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
