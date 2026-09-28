# OASIS XSLT 1.0 Greek Alphabetic Numbering

- Date: 2026-09-26
- Status: Verified semantic and corpus-frontier evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The unchanged `Lotus/numbering_numbering14` case requests the corpus-declared
discretionary Greek alphabetic numbering behavior through
`xsl:number format="α" letter-value="alphabetic"`. FastXSLT previously rejected
the format token as unsupported `FXST1049`.

## Bounded implementation

The private numbering representation now retains a Greek-lowercase alphabetic
token. Formatting uses the 25-codepoint lowercase Greek sequence from `α`
through `ω`, including final sigma in its Unicode/codepoint position, and the
same one-based rollover rule as the existing Latin alphabetic formatter:
`25 -> ω`, `26 -> αα`.

Number-format tokenization recognizes Unicode alphanumeric characters so the
Greek token is not mistaken for punctuation. The existing `letter-value`
validation admits this token only for `alphabetic`; no general locale,
traditional-numbering, or arbitrary Unicode alphabet facility is inferred.

The case also exposed a physical serialization issue. HTML output encoded as
ISO-8859-1 must use numeric character references for result text outside that
encoding. The bounded single-byte HTML lane now reuses the XML-aware lexical
context encoder, while the text output method remains literal and continues to
reject unrepresentable characters.

## Corpus observation

The unchanged case now initializes and executes successfully. Its archived
comparison method is HTML, for which the local harness has not admitted a
semantic comparator, so it is not credited as an exact pass.

- exact expected-result matches remain **2,153 / 3,173 (67.85%)**;
- initialized cases rise from 2,363 to **2,364**;
- successful executions rise from 2,312 to **2,313**;
- initialization failures fall from 807 to **806**;
- execution failures remain **51**; and
- the `FXST1049` standard-operation frontier falls from three cases to **two**.

This is implementation evidence for one explicitly discretionary behavior, not
a claim that every Greek, locale-sensitive, or traditional numbering system is
supported.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
