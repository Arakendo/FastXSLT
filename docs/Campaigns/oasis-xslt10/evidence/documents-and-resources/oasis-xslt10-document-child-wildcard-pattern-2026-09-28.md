# OASIS XSLT 1.0 Document-Child Wildcard Pattern

- Date: 2026-09-28
- Status: Verified engine correction
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Related review: AR-0019

## Defect

The typed `MatchPattern::DocumentElement` representation was shared by the
XSLT 1.0 `/*` pattern and XSLT 3.0 `document-node(element(...))`. Those forms
have different candidate nodes: `/*` matches the document element, while the
XSLT 3.0 pattern matches the document node after testing its child. A match
pattern must instead select the document-element node itself: the candidate is
an element whose parent is the document node.

The same representation assigned `/*` the `-0.5` node-test default priority.
Because `/*` is a multi-step location path pattern, its XSLT 1.0 default
priority is `0.5`. The wrong priority let a later `node()` rule suppress a
document-element rule.

## Correction

The private plan now distinguishes `DocumentChildElement` from
`DocumentElement`. The former requires an element candidate with a
document-node parent and uses the existing path default priority. The latter
retains the already evidenced XSLT 3.0 document-node semantics and priority.
One compact runtime regression places a later `node()` rule after `/*`, proving
that the document element is selected by the intended rule.

No generic provider, alternate matcher, or unbounded traversal was introduced.
The parent observation is charged through the existing XPath node-visit budget.

## Corpus effect

Microsoft `AttributeSets__91043` remains exact under the corrected semantics.
Microsoft `Keys_PerfRepro2` no longer falls through built-in templates and emits
the source string value. It reaches the stylesheet's real body and then reports
the next explicit unsupported boundary: a source-dependent numeric expression
inside `format-number()`.

The strict lower bound remains 2,261 / 3,173 (71.26%). Initialized cases remain
2,440. Successful executions move from 2,392 to 2,391 and classified execution
failures from 48 to 49. The visible mismatch frontier becomes zero; no incorrect
output is credited. The independent broad HTML comparator gap remains one.

## Verification

```powershell
cargo test -p fastxslt --all-features document_child_wildcard_pattern_matches_the_document_element
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Keys_PerfRepro2#1'
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
