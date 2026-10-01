# OASIS XSLT 1.0 typed-ID count composition evidence

Date: 2026-09-30

## Question

Can XSLT 1.0 `count()` reuse FastXSLT's existing typed-ID lookup semantics
without adding a second ID implementation or widening the private DTD profile?

## Change

The value-expression compiler now recognizes `count(id(...))` in XSLT 1.0
compatibility mode and retains the existing typed-ID lookup as its argument.
Runtime evaluation invokes the same charged lookup used by direct `id()` and
counts its unique document-ordered result. Prepared-program retention accounting
also charges the retained lookup exactly as it does for direct `id()`.

The focused bounded-source regression uses an IDREF-valued node-set argument
containing `third first third`. Direct iteration produces `AC`, and the new
count produces `2`, proving shared node-set conversion, uniqueness, and document
order rather than a string-specific shortcut.

No XML grammar, DTD acquisition, XDM representation, authority, or production-
default DTD policy changed.

## Conserved OASIS result

The hash-verified 3,173-case OASIS CD04 sweep reports:

- 2,454 exact comparisons (77.34%), up from 2,451;
- 2,648 initialized cases, up from 2,645;
- 2,602 successful executions, up from 2,599;
- 522 initialization failures, down from 525;
- 46 execution failures, unchanged;
- ten visible mismatches and four comparator gaps, unchanged; and
- 423 / 431 expected-error credit, unchanged.

The three exact gains are `Lotus/idkey_idkey61#1`,
`Lotus/idkey_idkey62#1`, and `Lotus/idkey_idkey63#1`.

## Attribution correction

Those three cases previously failed while compiling a stylesheet expression,
so failure-location-based direct-DTD accounting could not attribute their
principal-source DTDs. Successful execution now exposes them. The observed
direct-DTD attribution set therefore changes from 117 to 120 cases:

- 18 bounded internal-subset parses;
- 93 bounded sealed-external-subset parses; and
- nine explicit unsupported declaration-semantics outcomes.

This is improved observation, not a newly discovered parser feature and not a
reopening of AR-0025. The complete 3,173-case catalog denominator remains the
conserved compatibility denominator; the direct-DTD attribution set is an
evidence inventory that can grow when earlier blockers are removed.
