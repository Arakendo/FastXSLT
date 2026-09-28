# OASIS XSLT 1.0 DOCTYPE Reference Inconsistencies

- Date: 2026-09-27
- Status: Verified archival disposition evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Bounded DOCTYPE comparison made six long-hidden output differences visible.
Review of the unchanged stylesheets and references shows that the references
cannot be used as semantic pass/fail oracles without requiring behavior that
contradicts their own result trees or the XSLT 1.0 HTML output rule.

## Dispositions

The following Lotus references hard-code `HTML` as the DOCTYPE name even when
the first result element is `root`:

- `Lotus/output_output40`; and
- `Lotus/output_output48`.

XSLT 1.0 requires the HTML DOCTYPE name to be the name of the first result
element. FastXSLT therefore emits `root` and does not reproduce the archival
name.

`Lotus/output_output59` constructs a top-level processing instruction before
the document element. Its reference places the DOCTYPE first and the
processing instruction between the DOCTYPE and element, despite the HTML
output rule requiring the DOCTYPE immediately before the first element.
FastXSLT retains the constructed processing instruction before that DOCTYPE.

The three Microsoft references below hard-code `html` as the DOCTYPE name for
an `<out>` result and omit the `foo` text explicitly selected by their shared
stylesheet template:

- `Microsoft/Output_DoctypePublicAndSystemAttribute`;
- `Microsoft/Output_DoctypePublicAttribute`; and
- `Microsoft/Output_DoctypeSystemAttribute`.

Reproducing either difference would regress correct result construction or
DOCTYPE naming. All six cases therefore receive exact, test-guarded
`unusable-reference-result` dispositions and no pass credit.

## Corpus result

The conserved sweep remains **2,172 / 3,173 exact matches (68.45%)**, with
2,371 initialized and 2,326 successfully executed. Visible mismatches fall
from nine to **three**; unusable archival reference exclusions rise from 48 to
**54**. The remaining mismatches are the broad Microsoft HTML stress case and
two numeric expectations already annotated with upstream doubts.

Comparator gaps remain 52, execution failures remain 45, and expected-error
credit remains 423 / 431.

## Boundaries

This classification does not change engine semantics, weaken DOCTYPE
comparison, normalize arbitrary HTML results, or convert an exclusion into
conformance credit.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
