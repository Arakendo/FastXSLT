# OASIS XSLT 1.0 Following-Sibling Text Kind Test

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can the shared location-path evaluator admit `following-sibling::text()` with
the same charged axis semantics as existing sibling node tests, while leaving
the separate XSLT whitespace-policy boundary explicit?

## Method

- Add a typed following-sibling text step beside the existing named-element,
  any-element, and any-node steps.
- Enumerate only later children of the same parent, charge candidate visits,
  retain text nodes only, and preserve forward-axis document order.
- Add a focused mixed-node regression.
- Run unchanged Microsoft `Whitespaces__91449` through `__91452` from the
  hash-verified local OASIS XSLT 1.0 CD04 archive.

## Result

The two `xsl:preserve-space` cases (`__91449` and `__91450`) exactly match
their expected XML. The two `xsl:strip-space` cases (`__91451` and `__91452`)
now initialize and reach the existing explicit `FXRT1014` boundary because
their sources contain `xml:space`; they are not reported as passes.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,107 | 2,111 | +4 |
| Initialization failures | 1,028 | 1,024 | -4 |
| Executed successfully | 2,020 | 2,022 | +2 |
| Execution failures | 87 | 89 | +2 |
| Exact XML-semantic matches | 1,889 | 1,891 | +2 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,891 / 3,173 = 59.60%`.
The generic `FXXP1001` initialization frontier falls from 39 to 35 cases.
This is compatibility evidence against a hash-verified, non-redistributed
archive; it is not a broad conformance claim.

## Boundaries

- Only the text kind test is newly admitted on the following-sibling axis.
- Following-sibling comment and processing-instruction kind tests remain
  unsupported.
- This change does not broaden the accepted strip-space profile or bypass the
  `xml:space` guard established by ADR-0012.
- Sibling enumeration remains invocation-local, cancellable, and work charged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features following_sibling_text_kind_test_selects_only_text_nodes
./scripts/measure-oasis-xslt10.ps1 -TraceCase Whitespaces__91449
./scripts/measure-oasis-xslt10.ps1 -TraceCase Whitespaces__91450
./scripts/measure-oasis-xslt10.ps1 -TraceCase Whitespaces__91451
./scripts/measure-oasis-xslt10.ps1 -TraceCase Whitespaces__91452
./scripts/verify.ps1
```
