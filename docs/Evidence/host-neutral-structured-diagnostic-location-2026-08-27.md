# Host-Neutral Structured Diagnostic Location

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Boundary | Private host-neutral workbench facade |
| Case | Unsupported `xsl:message` during stylesheet compilation |
| Code/category | `FXST1006` / `unsupported` |
| Outcome | Owned logical resource and byte span survive translation |

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

## Claim boundary

This is one private facade translation, not a stable public diagnostic schema
or catalog. XML preparation failures still retain only identity plus debug
detail at this boundary. The isolated-worker protocol and native binary envelope
do not yet serialize the new location, so cross-host parity remains open.
Related locations, bounded diagnostic collections, disclosure policy, causes,
unknown-code handling, and reportable semantic outcomes also remain unresolved
under AR-0004.
