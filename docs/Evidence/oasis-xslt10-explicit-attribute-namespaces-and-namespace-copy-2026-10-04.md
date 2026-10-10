# OASIS XSLT 1.0 Explicit Attribute Namespaces and Namespace Copy

The 2026-10-04 continuation repairs computed-attribute namespace semantics
and admits bounded direct source namespace copying through existing owners.
Exact comparisons increase from 2,485 to 2,488 of 3,173 (78.41%). The copy
lowering separately advances one case without increasing the exact numerator.

## Scope and reproduction

The unmodified OASIS XSLT 1.0 CD04 archive has SHA256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
Replay uses `scripts/measure-oasis-xslt10.ps1`, including traces for
`Lotus/attribset_attribset23`, `Microsoft/Attributes__78373`,
`Microsoft/Attributes_SpecExample`, and `Microsoft/Namespace_Bug85356`.
No fixture, reference output, comparator, exclusion, production parser policy,
authority, vendor gitlink, or public API changes are part of this tranche.

## Explicit attribute namespace semantics

[XSLT 1.0 section 7.1.3](https://www.w3.org/TR/1999/REC-xslt-19991116#creating-attributes)
defines an explicit attribute namespace using the lexical QName's local part
and supplied URI. The prefix is presentation rather than expanded identity.
Its reserved-prefix example explicitly permits discarding `xmlns` when an
ordinary namespace is supplied. Previously both static and dynamic resolution
rejected that valid case; rebound `xml` and prefixed names with explicit empty
namespaces also failed too early.

Static and dynamic XSLT 1.0 resolution now retain the local name and explicit
namespace, letting the existing serializer choose a safe binding. Empty
overrides produce null namespaces; the XML namespace uses canonical `xml`.
Runtime reuses the existing per-attribute compatibility flag derived from
the instruction's static context, not the principal stylesheet version or
host policy. Lexical QName validation, exact-name `xmlns` rejection, reserved
XMLNS URI rejection, absent-override resolution, and modern rules remain intact.
Existing dynamic name/namespace evaluation and resource charge points remain
shared. One prior hand-authored test had incorrectly expected an error for
the normative reserved-prefix example; it now asserts the valid expanded name.

Fifteen computed-attribute tests pass. Added controls cover static, literal
AVT, and source-path names, discarded reserved and unbound prefixes, ordinary,
empty, and XML namespace overrides, safe output reparsing, and profile-local
rejection. A mixed-version control tests both directions of local versus
principal version selection; the modern literal AVT remains explicitly
unsupported rather than inheriting compatibility behavior. The unchanged `Lotus/attribset_attribset23#1`,
`Microsoft/Attributes_SpecExample#1`, and `Microsoft/Attributes__78373#1`
are the newly exact cases. Archival prefix spelling is not made engine identity.

## Direct namespace copy lowering

The existing private namespace compiler accepts a terminal source namespace
path and lowers `xsl:copy-of` to a qualified selection with an empty `xsl:copy`
body. Namespace nodes have no children or attributes, so this reuses full copy
semantics for the admitted node kind. Mixed unions and document selections
remain unsupported rather than becoming shallow ordinary tree copies.
Quoted axis-looking text does not activate this path. The synthetic loop and
copy retain the original `xsl:copy-of` location.

Two focused controls compare direct and explicit-loop copying under both
profiles, complete-reference and visibility-view sources, exact and one-less
navigation/result-byte budgets, cancellation, recovery, and late attachment
diagnostics. No new semantic node, eager arena population, result type, or
runtime interpreter is introduced.

`Microsoft/Namespace_Bug85356#1` now initializes, then reports `XTDE0440`
when attaching a default namespace to an unnamespaced result element.
The broader XSLT 1.0 recovery choice remains deferred under
[ADR-0021](../ADR/ADR-0021-derived-qualified-namespace-node-occurrences.md).
This is an initialization gain, not an exact pass.

## Accounting and remaining work

Full replay reports 2,689 initialized, 481 initialization failures, 2,640
successful executions, and 49 execution failures. Exact XML comparisons are
2,488; fourteen mismatches and four comparator gaps remain visible. Expected
error credit remains 423 / 431 and direct-DTD attribution remains 134 cases.
None of those observations is added to the exact numerator. Reaching 80%
requires 2,539 exact comparisons, another 51 cases.

Prefixed computed elements with explicit empty namespaces remain a separate
compatibility gap. General namespace copy recovery, namespace-valued variables,
cross-document value identity, and secondary-document DTD expansion are not
implicitly admitted to reach a percentage target.

## Ownership and validation

Following [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md),
namespace lowering belongs to the existing private namespace compiler. Its
parent adds only dispatch, and computed-attribute resolution stays with its
existing compiler/runtime owners. No host policy or runtime state enters
compilation. The large instruction compiler remains a decomposition pressure
point; this focused change does not claim that debt is resolved.

Focused tests and the full corpus replay pass. Workspace formatting, strict
Clippy, tests, documentation, conformance inventories, unsafe-surface checks,
and local Markdown links pass. The official `wasm32-unknown-unknown` adapter
passes its all-features check. No performance improvement or broad XSLT 1.0
conformance claim is made.
