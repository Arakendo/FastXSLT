# OASIS XSLT 1.0 `contains()` over Typed `concat()` Operands

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can XSLT 1.0 `contains()` compose two already supported dynamic `concat()`
operands without introducing a general expression evaluator?

## Implemented boundary

The XSLT 1.0 value compiler now recognizes the exact two-argument shape
`contains(concat(...), concat(...))`. Both operands compile to the existing
typed concat representation, including its bounded argument count, source
context, variable lookup, path evaluation, and work charging. Runtime evaluates
the two strings and charges the containment test before emitting its boolean
lexical result.

The change does not add general nested function composition, change modern
XPath behavior, or create another string evaluator. An equivalent version 3.0
expression remains explicitly unsupported by the current modern slice.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,238 to 2,239;
- successful executions rise from 2,178 to 2,179;
- exact XML comparisons rise from 2,029 to 2,030 / 3,173 (63.98%);
- unchanged `Lotus/string_string57#1` now produces the expected
  `<out>true</out>` result; and
- execution failures remain 60, comparison mismatches remain 66,
  expected-error unexpected successes remain five, and no execution panic
  occurs.

## Verification

A focused compiler test proves that only the XSLT 1.0 profile selects the typed
composition. A focused runtime test evaluates a context-derived first concat
operand and a literal second concat operand. The unchanged corpus case then
exercises the same path through compilation, execution, serialization, and
XML-semantic comparison.

The archive remains local and unmodified. This is compatibility evidence, not
a broad conformance claim.

## Source identity composition on 2026 10 04

The bounded XSLT 1.0 concatenation plan now carries an ordinary source-path
identity part. It reuses the controlled location-path selector and existing
source-node identity formatter, rather than introducing another identity
representation. The zero-argument form selects the current source node, empty
selection contributes an empty string, and multiple selected nodes use the
first in document order as required by
[XSLT 1.0 section 12.4](https://www.w3.org/TR/1999/REC-xslt-19991116#function-generate-id).
This does not change the stricter cardinality behavior of the existing modern
scalar identity evaluator. Namespace-ID strings, temporary identity parts,
nested concatenations, and modern composition remain unsupported. Key-use and
global concatenation eligibility remain unchanged.

Each identity part charges one XPath operation before controlled path
selection. The existing containment and output charges remain in force.
Compiled retention now includes the identity path's owned capacity.

Two new compiled runtime controls test empty/first-node selection, repeated
identity, distinct document/element identity, zero-argument/current-node
agreement, visibility-view versus complete-reference parity, exact and
one-less XPath budgets, mid-evaluation cancellation, short-circuit charge
parity, and reuse after failure. All five composed-contains tests pass.

The complete unchanged OASIS CD04 replay conserves 3,173 cases, 2,684
initialized cases, 2,637 successful executions, 486 initialization failures,
47 execution failures, 2,485 exact XML comparisons (78.32%), fourteen XML
mismatches, four comparator gaps, and 423 / 431 expected-error credits.
The archive SHA256 remains
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
`Lotus/idkey_idkey31#1` progresses from `generate-id(/)` inside concatenation
at stylesheet span 1576..1645 to its filtered namespace-ID variable expression
at span 3408..3517. It remains initialization/unsupported `FXXP1001` and earns
no new exact credit. No fixtures, comparator policy, or exclusions changed.

ADR-0004 review: the value compiler remains a pressured 3,671-line owner with
twelve direct sibling files. This checkpoint adds only five production lines
to its existing bounded concat-family dispatch. Retain this local extension
without changing module visibility or dependency direction; broader string
composition work must first review extraction of that family into a private
typed compiler owner. The 188-line composed-contains test family remains
cohesive. No public API, host authority, prepared representation, wire format,
or unsafe surface changes.

Workspace formatting, strict Clippy, tests, documentation, conformance
inventories, and local Markdown links pass. The official
`wasm32-unknown-unknown` adapter also passes its all-features build check.

The subsequent same-day
[ID name composition and decomposition checkpoint](oasis-xslt10-id-name-composition-and-decomposition-2026-10-04.md)
performs the private concat/ID compiler extraction required before further
composition, adds shared typed-ID name consumers, and repairs the generated-ID
lexical form. The counts and line sizes above describe the earlier checkpoint,
not the later state. Exact coverage remains unchanged.
