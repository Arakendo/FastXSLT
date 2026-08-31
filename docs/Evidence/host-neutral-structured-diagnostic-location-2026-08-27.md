# Host-Neutral Structured Diagnostic Location

| Field           | Value                                                                                         |
| --------------- | --------------------------------------------------------------------------------------------- |
| Date            | 2026-08-27                                                                                    |
| Boundary        | Private host-neutral workbench facade                                                         |
| Cases           | Malformed source XML; unsupported `xsl:message` during stylesheet compilation                 |
| Code/categories | `FXXM0002` / `invalid`; `FXST1006` / `unsupported`                                            |
| Outcome         | Owned logical resource and byte span survive facade, native, and isolated ASP.NET translation |

## Executed boundary

The focused compiler failure already owns a private `SourceLocation`. Runtime
translation now retains that value separately from its display detail, and the
workbench facade projects it as an owned `WorkbenchLocation` containing:

- logical resource identity;
- inclusive start byte offset; and
- exclusive end byte offset.

The existing negative workbench case reads those three fields directly and
verifies `urn:fastxslt:diagnostic:unsupported-stylesheet:103..117`. It does not
parse the human-readable detail to recover provenance. The location is inert
owned data and cannot reopen the original resource.

The XML parser now exposes its owned failure span separately from its private
failure representation. Preparation retains that span with the logical source
identity, and the facade projects malformed `<order></other>` at
`urn:fastxslt:diagnostic:malformed-source:7..7`. The point span records the
parser's exact known offset without inventing a token width. Span-bearing XML
failures retain their full parser-owned range; missing-root failures use the
document start `0..0`. Cooperative-control failures remain invocation failures
without a source location.

## Claim boundary

The isolated-worker protocol and native binary envelope now carry optional
resource, start, and end fields in the same order. Their managed decoders expose
one shared `FastXsltDiagnosticLocation` shape. The explicitly unstable native
ABI version advances from 0 to 1, and a native unit verifies the exact
`FXST1006` envelope fields. The managed project builds against both decoders.

An end-to-end release-mode probe launched the ASP.NET 8 workbench and called
`POST /experiment/diagnostic-parity`. The isolated worker returned HTTP 200 with
the exact structured locations for both `FXXM0002 / invalid` and
`FXST1006 / unsupported`, with null request identities. The worker PID remained
unchanged and a following transform returned the expected `for-004` result, so
transferring the diagnostics did not poison reusable state.

This remains a private workbench translation, not a stable public diagnostic
schema or catalog. Related locations, bounded diagnostic collections,
disclosure policy, causes, unknown-code handling, and reportable semantic
outcomes remain unresolved under AR-0004.
