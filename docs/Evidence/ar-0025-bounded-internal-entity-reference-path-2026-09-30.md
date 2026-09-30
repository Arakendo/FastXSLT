# AR-0025 Bounded Internal-Entity Reference Path

## Purpose

Record the first executable AR-0025 experiment without changing FastXSLT's
production DTD policy or claiming general DTD support.

## Implemented reference slice

The XML boundary now contains a private safe-Rust parser for a deliberately
narrow internal-subset profile. An explicitly selected test-only path can:

- parse bounded internal general-entity declarations;
- resolve forward and nested entity references;
- expand declared entities in ordinary text and attribute values;
- preserve the authored entity-reference span on emitted text;
- charge declaration and expansion work through the existing XML work domain;
  and
- stop before XDM construction on malformed, cyclic, unsupported, cancelled,
  or over-budget work.

Four limits are independent: declaration count, nesting depth, reference count,
and cumulative replacement bytes. The reference and replacement-byte counters
include retained declaration expansions and later document use rather than
counting only the admitted source bytes.

## Preserved denial boundary

The ordinary `parse_document` and `parse_document_controlled` entry points still
select complete DTD denial. The bounded path must be chosen explicitly inside
the crate's test configuration and is not exposed by the Rust facade, .NET
adapters, isolated worker, or WASM adapter.

The experimental parser rejects rather than ignores:

- `SYSTEM` and `PUBLIC` identifiers;
- external general or parameter entities;
- parameter entities;
- attribute-list and default-attribute declarations;
- notation and unparsed-entity declarations;
- element or other declaration kinds;
- replacement text that can inject markup; and
- entity-bearing namespace declaration values that the current tokenizer tries
  to resolve before the adapter receives the start event.

No resolver, file open, network request, catalog lookup, or snapshot mutation
was added. A URL-shaped system identifier is reported as unsupported by the
private experiment and remains forbidden by the normal parser path.

## Verification

Focused tests cover nested and forward expansion, character references, text
and attribute use, cycles, depth, replacement size, external identifiers,
parameter entities, declaration kinds, the unchanged default denial path, and
source-located failures. The complete `fastxslt` crate suite passes with 1,387
tests passed and 34 ignored. Strict Clippy passes for every crate target and
feature.

## Findings

- General internal entity replacement can be owned by FastXSLT without giving
  quick-xml or a lexical URI acquisition authority.
- The current tokenization boundary is sufficient for character-data and
  ordinary attribute replacement, but not for namespace declaration values or
  replacement text that produces markup.
- A useful corpus profile will require more than general entities. The OASIS
  inventory is dominated by external identifiers and attribute-list/typed-ID
  semantics, so this reference slice is not expected to unlock the next
  percentage point by itself.
- The exact 117-case direct frontier now has runner-owned accounting: 113
  internal subsets, 94 external identifiers, 41 attribute-list declarations,
  27 explicit default-attribute candidates, 17 entity declarations, 13
  internal general-entity candidates, five external general-entity candidates,
  and 11 typed-ID candidates. No direct-frontier input contains a parameter
  entity, notation, or unparsed-entity candidate. These properties overlap.
- Replaying the private reference parser against the exact failing resource for
  every direct case produces 117 explicit
  `unsupported-declaration-semantics` outcomes. The 30 stylesheet failures are
  28 principal modules plus two included/imported modules; the runner follows
  the structured/failure resource identity rather than incorrectly attributing
  those two failures to the principal stylesheet.
- Default attributes, typed IDs, notations, and unparsed entities remain
  observable XML/XDM semantics and must not be skipped to improve initialization
  counts.

## Non-claims

- Production DTD support is not enabled.
- No OASIS case receives pass credit from this experiment yet; the complete
  runner remains at 2,348 / 3,173 exact comparisons (74.00%).
- The experiment is not a validating XML processor.
- External subset resolution, typed IDs, default attributes, parameter
  entities, notations, and `unparsed-entity-uri()` remain unsupported.
- This evidence does not justify an XSLT 1.0 conformance statement.
