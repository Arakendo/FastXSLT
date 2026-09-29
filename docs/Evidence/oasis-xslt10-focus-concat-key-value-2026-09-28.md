# OASIS XSLT 1.0 Focus-Dependent `concat()` Key Value -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic slice and conserved corpus movement  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Microsoft `Keys_PerfRepro2` case groups nodes selected by one key
using a second key whose lookup value is
`concat(PlanLevelDescription, ':', BusTeamDescription)`. The existing typed
Muenchian plan and invocation-owned index reached this expression, but the
lookup-value compiler admitted only static values, variables, location paths,
or nested key calls.

## Change

An XSLT 1.0 `key()` lookup value may now reuse the existing bounded typed
`concat()` expression. Literal, variable, and location-path parts retain their
ordinary XSLT 1.0 string conversion, diagnostics, cancellation, and work
accounting. Location paths evaluate relative to the lookup call's current
focus, which is the current grouping candidate in the Muenchian plan.

The plan is stylesheet-derived immutable state. Evaluated values and the key
index remain invocation-owned; no state is shared across invocations,
prepared inputs, workers, snapshots, or generations. General XPath values and
a second key evaluator remain outside this slice.

## Verification

A focused compiler test proves that the Muenchian first-node lookup retains a
typed concat value. A focused runtime test groups a key-selected subset by a
second compound key, compares indexed execution with the complete charged-scan
oracle, and checks the exact serialized result.

The unchanged `Microsoft/Keys_PerfRepro2#1` case now initializes and executes
fully. Its archival expected result is an HTML/text reference that the current
XML comparator cannot parse as a document or fragment. The case therefore
moves from engine-unsupported to comparator-unsupported; no pass is inferred.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,423 | 2,424 | +1 |
| Executed successfully | 2,373 | 2,374 | +1 |
| Initialization failures | 747 | 746 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact XML expected-result matches | 2,213 | 2,213 | 0 |
| Visible comparator gaps | 52 | 53 | +1 |

The conservative exact-match ratio remains `2,213 / 3,173 = 69.74%`.
Expected-error credit remains 423 / 431.

```powershell
cargo test -p fastxslt --all-features xslt10_muenchian_key_group
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Keys_PerfRepro2#1'
./scripts/verify.ps1
```

## Normative references

- [XSLT 1.0 section 12.2, Keys](https://www.w3.org/TR/1999/REC-xslt-19991116#keys)
- [XPath 1.0 section 4.2, String Functions](https://www.w3.org/TR/1999/REC-xpath-19991116/#section-String-Functions)
