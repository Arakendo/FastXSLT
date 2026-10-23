# OASIS XSLT 1.0 Unicode Name Tests

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can the shared location-path evaluator match unqualified element names across
the XML 1.0 NCName character repertoire instead of imposing a private ASCII
restriction?

## Method

- Replace the path parser's ASCII-only name test with XML 1.0 Fifth Edition
  `NameStartChar` and `NameChar` ranges, excluding the colon required by
  NCName.
- Apply the same validation to ordinary path steps, qualified path components,
  axis predicates, and bounded path-boolean operands owned by the shared path
  evaluator.
- Retain expanded-name matching, namespace behavior, charged traversal,
  cancellation observation, and invalid-name rejection.
- Add a focused Unicode source/path regression and run unchanged Microsoft
  `Sorting__77530` and `Sorting__77982` from the hash-verified local OASIS XSLT
  1.0 CD04 archive.

## Result

The focused path selects the intended Unicode-named element. Both unchanged
sorting cases initialize, execute, and exactly match their expected results.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,113 | 2,115 | +2 |
| Initialization failures | 1,022 | 1,020 | -2 |
| Executed successfully | 2,024 | 2,026 | +2 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,893 | 1,895 | +2 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,895 / 3,173 = 59.72%`.
The generic `FXXP1001` initialization frontier falls from 33 to 31 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- This admits Unicode NCNames only where the shared location-path owner already
  admits an equivalent ASCII name test.
- It does not broaden the path grammar, add namespace inference, normalize
  Unicode names, or make a general XPath grammar claim.
- QName resolution and expanded-name matching retain their existing ownership;
  lexical colon handling is not folded into NCName validation.

## Reproduction

```powershell
cargo test -p fastxslt --all-features unicode_ncname_steps_match_unqualified_elements
./scripts/measure-oasis-xslt10.ps1 -TraceCase Sorting__77530
./scripts/measure-oasis-xslt10.ps1 -TraceCase Sorting__77982
./scripts/verify.ps1
```
