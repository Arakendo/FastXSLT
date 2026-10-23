# OASIS XSLT 1.0 `id()` number-pattern evidence

Date: 2026-09-30

## Result

The XSLT 1.0 `xsl:number/@count` compiler now admits a literal `id()` pattern
through the same bounded typed-ID lookup used by ordinary XPath evaluation and
template matching. Numbering evaluates the selected node set against the
principal source document, preserves document order, and counts only the
selected preceding siblings plus the current selected node.

The unchanged `Lotus/numbering_numbering91#1` case now initializes, executes,
and compares exactly. Its source declares typed IDs in a case-local
`iddata.dtd`; the OASIS measurement adapter admits that one reviewed subset
into the sealed snapshot and the existing AR-0025 path resolves it under the
same independent DTD and XML limits.

A focused bounded-DTD regression verifies the resulting sequence `(1)|(2)|`.

## Boundaries

- The pattern argument must be a literal string and cannot have a relative
  path tail. Broader function patterns remain unsupported.
- This adds no new DTD grammar, entity expansion, acquisition, or ambient
  authority. Production parsing continues to deny every DTD by default.
- The OASIS adapter recognizes only the named `numbering_numbering91` case and
  verifies its exact `SYSTEM "iddata.dtd"` declaration before admitting bytes.
- Runtime selection reuses the charged typed-ID selector and existing
  cancellation and work-budget behavior; it does not create a second ID index.

## Measurement

The complete hash-verified local OASIS CD04 sweep reports:

- 3,173 conserved catalog cases;
- 2,468 exact XML comparisons (77.78%);
- 2,665 initialized cases;
- 2,618 successful executions;
- 505 initialization failures;
- 47 execution failures;
- 12 visible XML mismatches;
- 4 comparator gaps;
- 423 / 431 expected-error credits.

The newly executable input raises direct DTD attribution to 133 cases: 30
bounded internal parses, 94 sealed-external parses, and nine explicitly
unsupported cases. This is improved observation after AR-0025's disposition,
not a wider DTD profile.
