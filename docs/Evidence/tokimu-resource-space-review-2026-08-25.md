# Tokimu Resource Space Review for FastXSLT

| Field | Value |
| --- | --- |
| Reviewed | 2026-08-25 |
| Source | `F:\LocalSource\tokimu` |
| Scope | Resource Space semantics and Weaver resolver boundary |
| Informs | AR-0003 |

## Material reviewed

- `AR-0009: Resource Store Identity And Kernel Boundary`
- `AR-0010: Weaver XSLT Resource Resolver Boundary`
- the Memory Resource Store implementation plan;
- the Weaver XSLT Resource Space consumer plan;
- public identity, address, immutable-content, registry, metadata, limits,
  query, mutation, and summary surfaces in `corpus/lib/resource-space`.

## Adopted lessons

- Store, root, folder, resource, and content identities answer different
  questions.
- Display names and normalized paths do not establish global identity.
- A content fingerprint is useful for diagnostics and candidate deduplication,
  but distinct logical resources may retain equal or shared immutable bytes.
- In-memory retention is a replaceable mechanism rather than universal semantic
  meaning.
- Entry, per-entry byte, and total-byte limits are caller policy applied by the
  provider.
- Format semantics own URI/reference interpretation; a resource store provides
  bounded lookup and must not become a generic URI parser.
- Explicit selected sessions prevent an XSLT engine from turning resolver access
  into ambient filesystem authority.

## FastXSLT adaptation

FastXSLT needs less folder/navigation behavior than Tokimu's general Resource
Space and stronger execution stability. AR-0003 therefore studies a mutable
resource-set builder that produces a sealed immutable snapshot. Stylesheet
compilation, source parsing, batches, and transformation graphs can reuse the
snapshot without binding the engine to filesystem paths or Tokimu's public
types.

No Tokimu code or API has been copied. The review supplies architectural
evidence; concrete FastXSLT identity, address, lifetime, cache, and batch types
remain open.

After this review, the project owner supplied an additional FastXSLT-specific
constraint: avoid retained or repeated file access because Windows Defender and
other security tools may contend with files, with prior Saxon behavior cited as
negative experience. ADR-0002 makes memory-resident execution binding; this
constraint is not inherited from Tokimu Resource Space.
