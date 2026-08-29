# Workbench Resource-Authority Diagnostic Parity

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Boundary | Unstable workbench Rust facade and .NET-mode diagnostic envelopes |
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

## Host-envelope parity

The native workbench's seven-field binary failure envelope and the isolated
worker's length-prefixed failure envelope are each tested with the real
compilation failures above. Both preserve code, category, resource location,
span fields, and detail without collapsing denial into absence. No new native
export, unsafe block, or unsafe allowance was introduced.

## Claim boundary

This proves diagnostic projection and encoding parity, not an end-to-end
managed resolver API. The existing native create export and isolated worker
initialize command still accept only one source and one principal stylesheet;
they cannot yet frame dependency sets or denial policy. The new Rust types are
documentation-hidden, feature-gated workbench inputs, not supported FastXSLT
facade contracts.

Catalogs, live callbacks, async acquisition, credentials, tenant policy, and
public disclosure defaults remain unselected under AR-0014.

## Validation

`scripts/verify.ps1` passes the enforced unsafe-surface check, formatting,
workspace Clippy with warnings denied, all-feature tests, Markdown links,
conformance-source checks and inventories, and workspace documentation. The
engine has 218 passing tests and 7 ignored manual probes; the native workbench
has 8 passes and the isolated worker has 1 transport-envelope pass.
