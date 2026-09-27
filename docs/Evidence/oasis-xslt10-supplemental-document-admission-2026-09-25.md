# OASIS XSLT 1.0 Supplemental-Document Admission

- Date: 2026-09-25
- Status: Verified corpus-accounting evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related reviews: AR-0014 and AR-0019

## Pressure

The local OASIS runner conserved all 3,173 catalog identities but stopped 35
cases before initialization whenever the catalog declared supplemental data.
Those cases therefore had a visible skipped disposition rather than an engine
frontier, even though the workbench already constructs a sealed in-memory
resource snapshot.

## Implemented harness slice

The local-only runner now reads each declared supplemental document from the
reviewed archive and admits its bytes under the same case-qualified logical
identity scheme used for stylesheet dependencies. The bytes enter the existing
bounded `ResourceSetBuilder` with the principal source and stylesheet before
the snapshot is sealed.

This is corpus-adapter admission, not ambient resource authority. Compilation
and execution still cannot open archive paths, files, or URLs. Missing
supplemental bytes retain a named infrastructure disposition.

## Measurement result

All 35 formerly skipped cases now reach engine initialization and expose their
next honest frontier. None yet reaches successful execution: the dominant
remaining work is typed `document()` selection/copy semantics, with a smaller
number of XML/parser and expression boundaries.

The complete sweep therefore remains at **2,329 initialized**, **2,282
successfully executed**, and **2,117 / 3,173 exact matches (66.72%)**. The
initialization-failure count rises from 806 to 841 because the 35 cases are now
classified by the engine rather than bypassed by the harness. The 69 visible
XML mismatches, eight host-parser-policy exclusions, and three
unusable-reference-result exclusions remain unchanged.

This tranche improves denominator disposition and establishes the sealed
resource prerequisite for later `document()` work; it does not claim support
for the function itself.

## Reproduction

```powershell
./scripts/measure-oasis-xslt10.ps1
```
