# OASIS XSLT 1.0 Namespace Identity Strings

The 2026-10-04 continuation admits a narrow namespace generated-ID string
consumer through the existing qualified selector. The unchanged OASIS CD04
replay remains 2,490 / 3,173 exact comparisons (78.47%): `idkey30` advances
past namespace identity strings but still fails at a later filtered union.
This is executable semantic progress, not a newly passing corpus case.

## Semantics and ownership

[XSLT 1.0 section 12.4](https://www.w3.org/TR/1999/REC-xslt-19991116#function-generate-id)
requires a stable distinct ASCII alphanumeric identifier beginning with a
letter, first-node conversion, and an empty string for an empty selection.
[ADR-0021](../ADR/ADR-0021-derived-qualified-namespace-node-occurrences.md)
defines namespace identity by origin, owning element, and prefix rather than
URI. The new scalar consumer uses that existing qualified selection and
normalization, without fabricated arena IDs or eager namespace storage.

Tree results preserve the existing principal-source ID formatter. Namespace
results use a disjoint tag, owner-element ID, and injective hexadecimal UTF-8
prefix encoding. Equal URI bindings with different prefixes or owners remain
distinct; Unicode prefixes cannot introduce non-ASCII generated identifiers.
Effective whitespace views and the complete derivation preserve identity.
Concrete spelling remains private and need not survive another transformation.

Local static context selects legacy first-node conversion versus modern
zero-or-one cardinality. XSLT 1.0 concat and containment reuse a typed qualified
identity operand, not a second expression evaluator. Compiled retention counts
the owned path. Global and key-use eligibility remain narrower. Temporary,
atomic, namespace-current, and secondary-document string consumers are not
admitted. The secondary-origin guard compares existing invocation-local
prepared origins, including effective views, and returns located unsupported
instead of labelling a secondary local ID as principal identity. This does
not settle document ranking or general cross-document identity.

## Resource and regression controls

Each function charges an XPath operation plus existing selector work. Namespace
encoding charges `64 + 2 * prefix UTF-8 bytes` XPath operations before allocating
the identifier; this is a conservative encoding-work charge, not RSS accounting.
Each cached dynamic origin inspected by the secondary-source guard charges a
node visit. Ordinary result text and serialization retain their existing charges.

Five focused compiled controls cover repeated identity, equal URI/different
prefix and owner, Unicode encoding, tree/namespace disjointness, empty and
mixed selections, first-node conversion, modern cardinality with provenance,
local version selection, view/reference parity, four concurrent invocations,
exact and one-less operation/visit/XDM/result-byte limits, cancellation,
recovery, concat/containment, temporary rejection, and sealed secondary-origin
rejection. The existing concat control now exercises its newly admitted
namespace operand rather than expecting the obsolete compiler rejection.
Hand-authored controls are not corpus credit.

## Cohesion review

The new test child owns namespace identity-string invariants, leaving the
namespace-focus test composition owner at 955 physical lines (952 before
its module declaration). The private production namespace-focus owner is
329 lines. The 13 direct files under the runtime sequence directory
trigger navigation review: retain these feature-named children at this
checkpoint; no stable new directory boundary is justified by one test family.
The pressured value compiler and typed conversion owner gain only routing
and operand dispatch, with no host policy, public abstraction, or backend.
The 3,520-line value compiler retains ordered typed-family routing while
namespace parsing and runtime selection remain in their existing private
owners. Retain that composition root at this checkpoint; this bounded change
does not complete its recorded decomposition debt.
Revisit grouping when an enduring namespace or expression subsubject needs
independent navigation; no build-time or performance improvement is claimed.

## Corpus reproduction and validation

Run `scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/idkey_idkey30'` against
the unchanged CD04 archive, SHA256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
The trace advances to `generate-id((following::node()|following::*/@*)[position() = $this])`.
No fixture, expected bytes, comparator, exclusion, vendor revision, DTD policy,
acquisition authority, public/ABI/wire contract, or unsafe surface changes.
All replay counters remain unchanged: 2,691 initialized, 479 initialization
failures, 2,642 successful executions, 49 execution failures, 14 mismatches,
four comparator gaps, and 423 / 431 expected-error credits.

The five focused controls pass. Final workspace formatting, strict Clippy,
tests, documentation, unsafe-surface and conformance-source checks pass.
Core tests report 1,578 passed and 57 ignored; native, WASM, and worker suites
report 22, four, and eighteen passed respectively. Local link verification
checks 1,975 Markdown files. The official `wasm32-unknown-unknown` all-features
check passes. The final post-guard full replay preserves every counter above,
with `idkey30` still explicitly unsupported at the later expression. No
performance, complete namespace support, or broad conformance claim follows.
