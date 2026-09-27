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

The standard-operation view also separates source documents rejected because
they contain a DTD, source documents not decodable as UTF-8, and other XML
failures. These are distinct host input/profile questions and must not appear
as one missing XSLT feature merely because they share `FXXM0002` at the adapter
boundary.

## First observation

Before the source-input split, the largest standard initialization frontier was
109 `FXXM0002` cases. Trace inspection showed that cluster was dominated by
DTD-bearing inputs and included non-UTF-8 archival inputs. The largest actual
XSLT/XPath implementation frontiers were therefore smaller and more specific,
including `disable-output-escaping`, language/case collation, and typed
location-path shapes involving functions.

This evidence does not classify DTD or legacy-encoding cases as conformance
passes or exclusions. It only prevents their input-policy pressure from
selecting unrelated semantic implementation work. Pass, failure, exclusion,
and conserved denominator totals are unchanged.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
