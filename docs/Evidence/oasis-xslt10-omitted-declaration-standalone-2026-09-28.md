# OASIS XSLT 1.0 Omitted Declaration and Standalone -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Two unchanged Microsoft output cases combine a canonical `standalone` value
with `omit-xml-declaration="yes"`. FastXSLT retained both settings and applied
the modern serialization rule that reports `SEPM0009` for the inconsistent
combination. XSLT 1.0 instead defines `standalone` as a value placed in the XML
declaration and requires no declaration when omission is requested.

One case expresses both properties on one declaration. The other composes them
across three same-precedence `xsl:output` declarations, so correcting only the
single-declaration parser would leave the merged behavior wrong.

## Change

The XSLT 1.0 compiler normalizes principal output settings before runtime:

- a standalone value on the same declaration is discarded when that
  declaration omits the XML declaration;
- same-precedence XSLT 1.0 output merging discards a retained standalone value
  once the effective merged settings omit the declaration;
- modern stylesheets retain both values and continue to receive the existing
  `SEPM0009` runtime validation.

This is a static compatibility rule. It does not weaken the modern serializer,
change byte encoding, or introduce a second serialization backend.

## Corpus result

Unchanged `Microsoft/Output__84617#1` now executes and compares exactly.
`Microsoft/Output__77940#1` also advances past `SEPM0009`, but remains visibly
uncredited: its archive result removes source whitespace copied by built-in
template processing, while the selected parser and XSLT semantics preserve
that whitespace. It receives the existing exact
`host-parser-policy-excluded` disposition rather than changing parser or
built-in-template semantics.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,431 | 2,431 | 0 |
| Executed successfully | 2,381 | 2,383 | +2 |
| Initialization failures | 739 | 739 | 0 |
| Execution failures | 50 | 48 | -2 |
| Exact XML expected-result matches | 2,220 | 2,221 | +1 |
| Visible XML mismatches | 9 | 9 | 0 |
| Host-parser-policy exclusions | 23 | 24 | +1 |
| XSLT-Output successful executions | 137 | 139 | +2 |

The conservative exact-match ratio is `2,221 / 3,173 = 70.00%`.
Only one of those two successful executions is an exact XML match.
Expected-error credit remains 423 / 431 and comparator gaps remain 53.

## Verification

A focused compiler test covers single-declaration normalization,
same-precedence XSLT 1.0 merging, and preservation of the modern runtime error
path. The unchanged archival cases then expose the exact pass and the later
whitespace mismatch separately.

```powershell
cargo test -p fastxslt --all-features xslt10_omitted_xml_declaration_discards_standalone_across_output_merging
./scripts/measure-oasis-xslt10.ps1 -TraceCase 84617
./scripts/measure-oasis-xslt10.ps1 -TraceCase 77940
./scripts/verify.ps1
```

## Normative references

- [XSLT 1.0 section 16, Output](https://www.w3.org/TR/1999/REC-xslt-19991116#output)
- [OASIS XSLT 1.0 Static Global Message](oasis-xslt10-static-global-message-2026-09-28.md)
