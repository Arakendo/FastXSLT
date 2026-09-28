# OASIS XSLT 1.0 HTML Public-Only DOCTYPE

- Date: 2026-09-26
- Status: Verified serializer and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The bounded DOCTYPE comparator exposed HTML results whose stylesheets supplied
`doctype-public` without `doctype-system`. The serializer returned early when
the system identifier was absent, so it silently omitted the public-only HTML
DOCTYPE required by the XSLT 1.0 HTML output method.

## Implementation

The serializer now treats a non-empty public identifier as sufficient to emit
an HTML DOCTYPE. It writes:

```text
<!DOCTYPE name PUBLIC "public-identifier">
```

when no system identifier is selected, while retaining the existing combined
PUBLIC/SYSTEM and SYSTEM-only forms. XML-like methods do not infer a
public-only declaration from this compatibility rule.

A focused test fixes the exact public-only serialization and retains bounded
identifier quoting.

## Corpus result

Two unchanged Lotus cases become exact:

- `Lotus/output_output17`; and
- `Lotus/output_output39`.

Four other previously visible mismatches now carry their requested DOCTYPE but
return to the named HTML-comparator gap because their remaining HTML bodies are
not XML-parseable. They receive no pass credit. This is a classification move,
not a semantic regression.

The conserved sweep changes from the immediately preceding comparator tranche:

- exact matches rise from 2,169 to **2,171 / 3,173 (68.42%)**;
- visible mismatches fall from 16 to **10**;
- comparator gaps rise from 48 to **52**;
- bounded matching-DOCTYPE passes rise from 8 to **10**; and
- initialization, execution, and expected-error totals do not change.

## Boundaries

This tranche does not establish a semantic HTML comparator, infer a default
HTML DOCTYPE, relax identifier validation, or alter production DTD input
policy.

## Verification

```powershell
cargo test -p fastxslt html_output_emits_a_public_identifier_without_a_system_identifier --all-features
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
