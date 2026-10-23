# OASIS XSLT 1.0 `id()` path-union evidence

Date: 2026-09-30

## Result

The existing XSLT 1.0 `xsl:apply-templates` union selector now composes
ordinary location paths with the shared charged typed-ID lookup. Every union
alternative is evaluated independently, then the combined node-set is
normalized to document order and duplicate nodes are removed before template
dispatch.

The unchanged `Lotus/idkey_idkey22#1` case now initializes, executes, and
compares exactly. The duplicated Microsoft case also initializes and executes
with the same semantic result, but remains a visible archival whitespace-result
mismatch rather than receiving exact-match credit.

A focused bounded-DTD regression selects the same three typed-ID elements in a
deliberately different union order and verifies document-order output `ABC`.

## Boundaries

- This adds no DTD grammar, entity, acquisition, or authority capability.
- It reuses the existing eight-alternative bound, typed-ID accounting, work
  charging, cancellation, document-order normalization, and duplicate removal.
- It does not add general function-call union operands. Only an already-admitted
  XSLT 1.0 `id()` lookup can occupy the new private union part.
- The new Microsoft mismatch remains visible; the engine does not alter
  stylesheet whitespace semantics to match one archival result.

## Measurement

The complete hash-verified local OASIS CD04 sweep reports:

- 3,173 conserved catalog cases;
- 2,466 exact XML comparisons (77.72%);
- 2,662 initialized cases;
- 2,615 successful executions;
- 508 initialization failures;
- 47 execution failures;
- 11 visible XML mismatches;
- 4 comparator gaps;
- 423 / 431 expected-error credits.

The two newly executable inputs raise direct DTD attribution to 132 cases: 30
bounded internal parses, 93 sealed-external parses, and nine explicitly
unsupported cases. This is improved observation after AR-0025's disposition,
not a wider DTD profile.
