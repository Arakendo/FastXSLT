# Sealed Stylesheet-Dependency Accounting

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Boundary | Private stylesheet include-graph preparation |
| Resource authority | One immutable `ResourceSnapshot` |
| Production profile | Maximum depth 1; 2 module occurrences; 1 MiB aggregate module bytes |
| Corpus conservation | XSLT30 `include-0401` remains selected/passed |
| Public contract | None selected |

> **2026-08-29 addendum:** the private production ceiling subsequently rose to
> three module occurrences and depth two at the same byte ceiling. The sibling
> capacity admits `include-0501`; the depth admits the fragment-selected,
> `xml:base`-qualified chain in `include-0103`. The original two-module,
> depth-one profile recorded below remains the profile under which this
> accounting experiment was first run.

> **Later 2026-08-29 addendum:** `include-0701` raised the private occurrence
> ceiling to five while retaining depth two and the 1 MiB byte ceiling. This
> admits exactly two included branches with one leaf import each; it does not
> select public graph limits.

## Experiment

The admitted-resource compiler now prepares the complete reachable
`xsl:include` graph before it compiles stylesheet semantics. Each module is
resolved relative to the containing module's qualified logical identity, read
only from the supplied sealed snapshot, counted, parsed into owned XDM, and
retained in a private tree. Semantic compilation begins only after graph
preparation succeeds.

The loader applies checks in an explicit order:

1. reject a reference whose edge depth exceeds the supplied depth limit before
   attempting acquisition;
2. resolve against the sealed snapshot and reject fragment selection;
3. reject an identity already on the active traversal path as a cycle;
4. charge the module occurrence before retaining it;
5. charge its bytes with checked addition before parsing; and
6. parse XML/XDM and discover the next direct include references.

This order prevents work beyond a depth limit, keeps cycle failure distinct
from count exhaustion, and ensures a byte-limit failure does not pay XML/XDM
construction cost.

## Executable controls

A three-module relative chain proves that identities and nesting survive graph
construction. Focused controls over the same immutable snapshot independently
produce:

- a depth-limit failure before the third resolution attempt;
- a module-occurrence failure after resolving but before retaining the third
  module;
- an aggregate-byte failure before parsing the second module; and
- an active-path cycle failure when the child points back to the principal.

The graph and its accounting state are local values returned only on complete
success. On failure Rust drops them, no `StylesheetProgram` exists, and the
snapshot has no mutation API. The resolver's bounded attempt counter is the
only intentionally consumed state. Existing missing-resource evidence likewise
continues to return structured `FXRS0002` without ambient fallback.

## Claim boundary

Module count currently means traversal occurrences, not distinct logical
identities or distinct byte bodies. That choice is useful for conservative
work accounting but is not selected as public policy. The experiment also does
not settle repeated-module semantics, shared subgraphs, `xsl:import`, import
precedence, fragments, `xml:base`, catalogs, live resolution, or caching.

At the time of this record, the real production slice was limited to the
principal plus one simplified included module. The deeper handcrafted graph
existed to verify accounting and failure behavior; it was not compiled or
reported as XSLT conformance. The addendum above records the later bounded
ceiling change.

## Validation

`scripts/verify.ps1` passes the unsafe-surface check, formatting, workspace
Clippy with warnings denied, all-feature tests, Markdown-link validation,
conformance-source cleanliness and inventory, XSLT30 metadata inventory, and
workspace documentation. The engine result is 216 passed with 7 ignored manual
probes; the native workbench adds 7 passes.
