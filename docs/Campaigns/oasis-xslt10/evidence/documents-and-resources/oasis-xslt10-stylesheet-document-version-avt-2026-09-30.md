# OASIS XSLT 1.0 stylesheet-document version AVT evidence

Date: 2026-09-30

## Current-status correction (2026-10-18)

The historical exact-match statement below is not the current result.
An unchanged full-corpus replay reports `Microsoft/Output__84025#1` as a
mismatch. Its current actual output includes the principal stylesheet's comment,
processing instruction, `p` element and included AVT test elements; the archive
instead contains a materially different `element`/`empty` fragment. Both
admitted stylesheet modules declare XML output, whereas the catalog purpose
says text output. The static stylesheet-version AVT is evaluated, but that does
not establish whole-case parity. The reason for the historical pass report has
not been established; do not use it as current conformance evidence or change
the reference to recover it. See the
[current mismatch audit](oasis-xslt10-literal-document-path-binding-2026-10-17.md#2026-10-18-mismatch-and-numeric-composition-audit).

## Result

The exact AVT expression `document('')/xsl:stylesheet/@version` is now folded
from the declaring stylesheet module's static XDM. It composes with the
existing XSLT 1.0 `current()/...` AVT path, so the unchanged duplicated AVT
fixtures initialize and execute without runtime stylesheet acquisition.

`Microsoft/Output__84025#1` now compares exactly. The duplicated
`Microsoft/BVTs_bvt005#1` executes with the same `current` and stylesheet-
version values but remains a visible archival whitespace-result mismatch.

A focused regression proves `current()/title` and the static stylesheet version
compose in one result attribute as `current: T, document: 1.0`.

## Boundaries

- This is exact static folding for one stylesheet-document introspection path,
  not general `document('')` execution.
- The authored `xsl` prefix must resolve to the XSLT namespace at the AVT site.
- No stylesheet bytes, parser objects, or prepared XDM are retained for runtime
  lookup, and no resource authority changes.
- Other stylesheet-document paths remain unsupported until independently
  justified.

## Measurement

The complete hash-verified local OASIS CD04 sweep reports:

- 3,173 conserved catalog cases;
- 2,467 exact XML comparisons (77.75%);
- 2,664 initialized cases;
- 2,617 successful executions;
- 506 initialization failures;
- 47 execution failures;
- 12 visible XML mismatches;
- 4 comparator gaps;
- 423 / 431 expected-error credits.
