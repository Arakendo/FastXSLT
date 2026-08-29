# Workbench Resource-Authority Diagnostic Parity

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Boundary | Unstable Rust facade plus native and isolated .NET initialization transports |
| Missing outcome | `FXRS0002 / missing-resource` |
| Denied outcome | `FXRS0003 / denied` |
| Authority order | Denial before snapshot membership disclosure |
| Public resolver contract | None selected |

## Experiment

The feature-gated workbench facade can now accept an explicit set of additional
stylesheet resources and a separate list of denied logical identities. All
bytes are bounded and copied into the same sealed snapshot before compilation.
The denial list is supplied to the snapshot resolver independently of admitted
membership; it is not inferred from missing bytes or encoded in a filename.

One positive control compiles and executes a principal stylesheet plus one
explicit simplified dependency. Two negative controls use the same principal
`xsl:include` reference:

- with no admitted dependency and no denial, compilation returns
  `FXRS0002 / missing-resource`;
- with the resolved identity denied but still unadmitted, compilation returns
  `FXRS0003 / denied`.

Both failures retain the principal stylesheet's structured include location
and mention the resolved logical dependency identity only in human-readable
detail. Callers select behavior from code and category, not detail parsing.

## Host-transport parity

ADR-0011 adds an explicit one-dependency initialization operation to each
unpublished .NET workbench mode. Both transport implementations now accept the
logical dependency identity, optional bounded bytes, and independent admission
and denial flags. Their positive controls compile and execute the included
module. Their negative controls preserve code, category, resource location,
span fields, and detail without collapsing denial into absence.

Malformed flag values and nonempty bytes attached to an unadmitted dependency
are rejected at the transport boundary. The managed adapters expose the same
narrow input and compile against native ABI version 2. The native extension
reuses the existing immediate buffer-copy helper: one export and one scoped
allowance were added, while the exact unsafe operation count remains two.

## Claim boundary

This proves one end-to-end workbench initialization shape, not a supported
managed resolver API. It deliberately accepts only one dependency and does not
select a general collection representation. The Rust and managed types remain
private or workbench-only inputs, not supported FastXSLT facade contracts.

Catalogs, live callbacks, async acquisition, credentials, tenant policy, and
public disclosure defaults remain unselected under AR-0014.

## Validation

Focused Rust transport tests pass with one admitted execution plus missing and
denied failures in each mode. The managed ASP.NET project builds in Release
configuration against ABI version 2. Full workspace gate results are recorded
at the implementation checkpoint rather than frozen into this evidence record.
