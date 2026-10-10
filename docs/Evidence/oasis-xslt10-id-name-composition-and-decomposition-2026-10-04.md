# OASIS XSLT 1.0 ID Name Composition and Decomposition

The 2026-10-04 continuation extends bounded typed-ID composition through
shared semantic consumers after private compiler/runtime extraction. It also
repairs the generated-ID lexical form. These are reusable semantics and
ownership improvements, not additional exact corpus passes.

## Scope and method

The local OASIS XSLT 1.0 CD04 archive remains unmodified, with SHA256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
Replay uses `scripts/measure-oasis-xslt10.ps1` with
`-TraceCase 'Microsoft/XSLTFunctions_Bug76054'` on the Windows workspace.
The catalog denominator remains 3,173. No fixture, exclusion, comparator,
production parser policy, or vendor gitlink changes are part of this tranche.

## Shared typed ID composition

The XSLT 1.0 compiler admits `name(id(...))`, `local-name(id(...))`, and
`namespace-uri(id(...))` through the existing typed-ID argument and relative
path grammar. Runtime reuses the shared charged ID selector and ordinary
node-name consumers, selecting the first node in document order and returning
empty text for an empty selection. Modern composition and non-source contexts
remain unsupported; no parser-owned types enter runtime values.

Two focused compiled controls exercise typed versus untyped ID attributes,
literal/path/variable arguments, duplicate and reverse-order tokens,
attribute/child continuation, empty lookup, namespace-aware name components,
visibility-view versus complete-reference parity, exact and one-less work
budgets, cancellation, diagnostic provenance, unbound variables, recovery,
and four concurrent invocations against the same immutable source/program.
Production parsing still rejects the DTD-bearing control input.

## Generated ID lexical repair

The existing source and temporary-root ID strings contained hyphens.
[XSLT 1.0 section 12.4](https://www.w3.org/TR/1999/REC-xslt-19991116#function-generate-id)
requires ASCII alphanumeric characters and an alphabetic first character.
The private formatters now use `fastxsltPrincipalN` and `fastxsltTemporaryD`
prefixes; the secondary-document root's existing `d` prefix already complies.
Distinct family prefixes and existing node/tree identity allocation remain
unchanged. No pointer address, resource URI, or cross-document ranking is
introduced. Result byte charges naturally follow the new shorter spelling.

Compiled controls assert lexical validity, repeated identity and source versus
temporary distinction. Existing generated-root corpus controls and source AVT
expectations remain in the regression suite. This repair does not admit
namespace-node ID strings or general mixed-document ID formatting.

## Decomposition review

Following [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md),
the pressured value compiler delegates bounded concatenation and typed-ID
compilation to private typed children. The runtime value evaluator delegates
typed-ID scalar consumers to a private child while selection stays in the
existing shared ID owner. No constructor traversal, runtime state, resolver,
host policy, public API, or alternate backend enters either compiler child.

The compiler parent is 3,516 lines, with a 91-line concat child and a
121-line ID child. Its fourteen direct sibling files trigger a directory
cohesion review: retain the named compiler owners together rather than create
arbitrary subfolders; review again when new responsibility clusters emerge.
The runtime parent is 3,096 lines and its ID child is 87 lines. These parents
remain pressured; this is a targeted extraction, not a claim that broader
decomposition is complete.

## Corpus disposition

Exact coverage remains 2,485 / 3,173 (78.32%). Initialized cases increase
from 2,684 to 2,685, initialization failures fall from 486 to 485, successful
executions remain 2,637, and execution failures rise from 47 to 48.
Four comparator gaps, fourteen visible mismatches, and 423 / 431 expected-error
credits remain unchanged. The 80% checkpoint still requires 54 exact gains.

`Microsoft/XSLTFunctions_Bug76054#1` now compiles its ID/name expression, then
fails with `FXXM0002` when secondary `document('xsltcfg.xml')` parsing encounters
`DtdForbidden` at span 0..35. The earlier private DTD experiment covered
principal inputs and stylesheet dependencies, not this dynamic-document path.
It is not extended implicitly. Direct-DTD attribution rises from 133 to 134
observed cases; that attribution is neither a pass nor a newly closed frontier.

## Validation

Focused ID/name, source identity, temporary/root identity, budget, cancellation,
and concurrent reuse tests pass. Workspace formatting, strict Clippy, tests,
documentation, conformance inventories, and local Markdown links are rerun
for this checkpoint and pass. Core tests report 1,564 passed and 57 ignored;
native tests report 22 passed and three ignored; WASM and worker tests report
four and eighteen passed respectively. Local link verification checks 1,972
Markdown files. The official `wasm32-unknown-unknown` adapter passes its
all-features check. No performance improvement is claimed from these changes.
