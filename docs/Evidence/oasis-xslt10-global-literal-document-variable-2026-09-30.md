# OASIS XSLT 1.0 global literal `document()` variable evidence

Date: 2026-09-30

## Question

Can a top-level XSLT 1.0 variable bind a literal `document()` result through
the existing sealed-resource mechanism, including when the declaration belongs
to an included stylesheet module?

## Result

Yes, for the narrow literal-reference and variable-rooted value-path slice.
The compiler retains the declaring module's logical identity as the static
base. At invocation initialization, the runtime resolves only against the
supplied sealed snapshot, parses the admitted bytes under the existing XML and
work limits, and stores the immutable prepared document in invocation-local
global state. Local frames can read or alias that global without moving it into
compiled state or introducing ambient acquisition.

A focused regression places the global declaration in
`modules/globals.xsl`, resolves `document('extra.xml')` relative to that module,
and reads `$external/doc/value`. The only available target is the sealed
`modules/extra.xml` resource.

The unchanged `Lotus/impincl_impincl08#1` case now initializes, executes, and
compares exactly. It exercises the same module-relative base behavior through
an included stylesheet and named template.

## Conserved boundary

- literal string argument only;
- no filesystem, network, parser-owned, or fallback acquisition;
- prepared external documents remain invocation-owned and reuse the existing
  bounded dynamic-document cache;
- XML/XDM construction, cancellation, budgets, denial, and missing-resource
  diagnostics are unchanged;
- compiled state retains only the stylesheet-derived logical reference and
  static base;
- no cross-invocation, cross-generation, or global cache is introduced.

The newly exposed `Lotus/mdocs_mdocs14#1` case is not credited. It requires
principal-source node ownership to remain valid while templates execute with
an external document as focus. Bare `NodeId` values do not establish that
cross-document association, so that larger multi-document ownership seam
remains explicit instead of being approximated.

## Measurement

The complete hash-verified local OASIS CD04 sweep reports:

- 3,173 conserved catalog cases;
- 2,464 exact XML comparisons (77.65%);
- 2,659 initialized cases;
- 2,612 successful executions;
- 511 initialization failures;
- 47 execution failures;
- 10 visible XML mismatches;
- 4 comparator gaps;
- 423 / 431 expected-error credits.

This is compatibility evidence, not a general XSLT 1.0 conformance claim.
