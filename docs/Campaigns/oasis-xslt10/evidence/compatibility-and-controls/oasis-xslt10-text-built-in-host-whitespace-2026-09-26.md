# OASIS XSLT 1.0 Text Built-In Host Whitespace

- Date: 2026-09-26
- Status: Verified named corpus disposition
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Related review: AR-0019

## Cases

- `Microsoft/Output__84010#1`
- `Microsoft/Output__84015#1`

Both stylesheets select the source `EscapeChars` element and invoke built-in
templates. The archival source contains whitespace-only text nodes between its
lowercase `char` children, while the stylesheet's explicit rule is for uppercase
`Char`. Standards execution therefore reaches the source whitespace through
built-in rules. FastXSLT produces that whitespace plus the character data; the
archival reference contains only the five characters.

The difference reflects the historical host parser's whitespace handling, not
a serialization comparator ambiguity and not permission for FastXSLT to drop
source whitespace globally. Both identities now carry the existing named
host-parser-policy disposition.

## Measurement effect

- exact expected-result matches remain 2,134 / 3,173 (67.25%);
- comparator gaps fall from 69 to 67;
- host-parser-policy exclusions rise from 21 to 23;
- zero XML mismatches remain.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
