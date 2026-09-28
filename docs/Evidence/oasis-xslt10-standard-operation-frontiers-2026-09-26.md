# OASIS XSLT 1.0 Standard-Operation Frontiers

- Date: 2026-09-26
- Status: Verified measurement refinement
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The original frontier ranking combined ordinary standard-operation failures
with cases whose catalog outcome is an error. That was useful for diagnosing
the engine, but it was a poor breadth queue: a large, correctly rejected
expected-error family could outrank missing behavior needed by successful
transforms.

## Measurement refinement

The local measurement now retains both views:

- complete initialization and execution frontiers diagnose every observed
  engine failure; and
- standard-operation frontiers omit catalog expected-error cases when ranking
  the remaining successful-transform backlog.

The standard-operation view also separates source and stylesheet documents
rejected because they contain a DTD, documents not decodable as UTF-8, and
other XML failures. These are distinct host input/profile questions and must
not appear as one missing XSLT feature merely because they share an XML
adapter diagnostic.

## First observation

Before the input splits, two large standard initialization frontiers were 109
`FXXM0002` source cases and 63 `FXXM0001` stylesheet cases. Trace inspection
produced the following bounded families:

- 85 DTD-bearing source documents;
- 9 non-UTF-8 source documents;
- 15 other source XML failures;
- 30 DTD-bearing stylesheet modules;
- 25 non-UTF-8 stylesheet modules; and
- 8 other stylesheet XML failures in the initial observation.

The later bounded historical-URI mapping exposed one previously unavailable
local ASP stylesheet and raised the final other-stylesheet-XML frontier to 9;
it did not change the DTD or non-UTF-8 families.

The 2026-09-27 BOM-selected UTF-16 adapter then moved 22 standard-operation
cases beyond the generic stylesheet-XML frontier. Eleven compile completely,
nine execute, and six compare exactly. The cases that do not complete now
report their actual numbering, runtime-formatting, or mismatch frontier rather
than an encoding-shaped parser failure. See the
[UTF-16 input evidence](oasis-xslt10-bom-utf16-input-2026-09-27.md).

The last stylesheet family contains representation-limit, namespace
well-formedness, and deliberately malformed-module pressure rather than one
shared XSLT feature. The largest actual XSLT/XPath implementation frontiers
are therefore smaller and more specific, including
`disable-output-escaping`, language/case collation, and typed location-path
shapes involving functions.

This evidence does not classify DTD or legacy-encoding cases as conformance
passes or exclusions. It only prevents their input-policy pressure from
selecting unrelated semantic implementation work. Pass, failure, exclusion,
and conserved denominator totals are unchanged.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
