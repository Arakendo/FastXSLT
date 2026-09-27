# OASIS XSLT 1.0 CDATA Whitespace Reference Boundary

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related decision: ADR-0012
- Related reviews: AR-0016, AR-0019

## Pressure

Four successfully executed Microsoft whitespace cases remained visible XML
mismatches after FastXSLT implemented the applicable strip/preserve declaration
precedence and inherited `xml:space` behavior:

- `Whitespaces__91443` and `Whitespaces__91444` expect 22 preceding text nodes,
  while the effective source view contains 21; and
- `Whitespaces__91455` and `Whitespaces__91456` expect seven spaces following a
  copied comment inside `span`, while the winning strip rule removes that
  whitespace-only text node.

All four differences originate in the same source fragment. The seven spaces
are written inside a CDATA section under `xml:space="default"`, and the source
comment describes them as "Always preserve CDATA whitespace."

## Disposition

CDATA delimiters are lexical XML markup, not a distinct XDM node kind or an
XSLT 1.0 whitespace-preservation authority. Their character content enters the
data model as text. The applicable `xsl:strip-space elements="*"` rule and
effective `xml:space="default"` therefore remove this whitespace-only text node
just as they remove equivalent character data written without a CDATA section.

The archival expected results retain a parser-specific CDATA-origin distinction
that FastXSLT's engine-owned XDM deliberately does not expose. The suite's own
`doubts.xml` also records that the reference count for `91444` appears too high.
Changing FastXSLT to reproduce these files would make whitespace semantics
depend on a lexical boundary that disappears during XML-to-XDM construction.

The four cases now carry exact `unusable-reference-result-excluded`
dispositions. They remain named in the conserved 3,173-case denominator and
receive no pass credit. This classification does not broaden the engine's
whitespace profile, change prepared XDM, or create a CDATA-specific text type.

## Measurement effect

The unchanged sweep remains at:

- 2,342 initialized cases;
- 2,291 successful executions; and
- 2,125 exact XML comparisons out of 3,173 (66.97%).

Visible XML mismatches fall from 31 to 27. Unusable-reference-result exclusions
rise from 32 to 36, and doubt-annotated visible mismatches fall from four to
three. No pass is inferred from an exclusion.

## Verification

The measurement adapter has an exact bounded-list test for all four identities.
The unchanged `91443` trace reports 21 versus the archival 22, and the retained
safe derived-document implementation and ADR-0012 visibility view continue to
agree on the effective source relationships.

```powershell
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1 -TraceCase Whitespaces__91443
./scripts/verify.ps1
```
