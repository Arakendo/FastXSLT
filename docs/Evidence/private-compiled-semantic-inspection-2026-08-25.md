# Private Compiled Semantic Inspection

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Implementation | `xslt::semantic_inspection_experiment` |
| Review | AR-0005 semantic inspection and explainability boundary |
| Claim | Private bounded semantic projection; no public or serialized schema |

## Projected questions

The first caller-shaped projection answers only questions supported by the
implemented compiled slice:

- Which logical stylesheet identity is being described?
- Which version did the stylesheet declare?
- Which output method and XML-declaration choice were compiled?
- How many root and exact-name templates exist?
- How many instructions exist recursively?
- Which implemented semantic instruction families occur, and how often?

The current feature vocabulary is literal result elements, literal text,
`xsl:value-of`, and `xsl:apply-templates`. These are semantic categories rather
than Rust enum names in a serialized contract.

## Bounds and ownership

The caller supplies explicit limits for copied text bytes and returned feature
kinds. Limit failures are structured and produce no partial report. Counts use
checked arithmetic.

The projection owns its strings and observations. A test drops the compiled
program before asserting the complete report, proving that it contains no
borrowed parser, XDM, instruction, location, or arena state. Comparing a clone
of the program before and after inspection proves report construction is
semantically inert for the implemented representation.

## Excluded state

The report contains no:

- source text, filesystem path, URI resolution, or resource bytes;
- node identity, source span, parser event, XDM handle, AST, instruction tree,
  optimizer IR, allocation address, or cache key;
- literal text/value content, selected path representation, or matched element
  names;
- invocation parameters, result data, messages, runtime budgets, or tracing;
  or
- JSON field names, schema version, ABI, managed type, or public Rust type.

Inspection performs no resolver call, filesystem access, extension execution,
or transformation. The test compiles from embedded in-memory fixture bytes and
then invokes a pure projection over the completed program.

## Limitations

The stylesheet identity is supplied by the private caller rather than owned by
the current compiled artifact. The selected standards profile, dependency
requirements, static diagnostics, capability requirements, redaction policy,
dynamic invocation summaries, compatibility/versioning, and ASP.NET caller
fields remain unresolved. The shape exists to gather those requirements, not
to stabilize them prematurely.
