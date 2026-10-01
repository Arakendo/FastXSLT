# OASIS XSLT 1.0 literal typed-ID match-pattern evidence

Date: 2026-09-30

## Question

Can literal XSLT 1.0 `id()` match patterns reuse the existing typed-ID lookup
without creating a second selector, broadening DTD authority, or evaluating the
document-rooted expression once per template candidate?

## Implementation

The previously local typed-ID selector is now one private runtime module shared
by value expressions, apply selection, and template matching. It retains the
same charged behavior:

- literal or node-set argument conversion;
- XML-whitespace tokenization;
- typed-ID index lookup;
- document-order normalization and duplicate removal; and
- optional relative location-path evaluation.

The match compiler admits only string-literal `id()` arguments as required by
the XSLT 1.0 pattern grammar. The selected nodes are retained in the existing
bounded invocation-owned document-rooted membership cache for the current
compiled template. No state is shared across invocations, sources, snapshots,
workers, or generations, and the complete charged selector remains the cache-
miss oracle.

The focused bounded-source regression composes direct `id()`, `count(id())`, an
IDREF-valued argument, relative attribute selection, and an
`id('third')/@value` pattern. A non-literal pattern argument is rejected as an
invalid XSLT 1.0 pattern.

No DTD parser, acquisition, XDM, host-authority, or production-default denial
behavior changed.

## Conserved OASIS result

The hash-verified 3,173-case OASIS CD04 sweep reports:

- 2,460 exact comparisons (77.53%), up from 2,454;
- 2,654 initialized cases, up from 2,648;
- 2,608 successful executions, up from 2,602;
- 516 initialization failures, down from 522;
- 46 execution failures, unchanged;
- ten visible mismatches and four comparator gaps, unchanged; and
- 423 / 431 expected-error credit, unchanged.

The six exact gains are:

- `Lotus/idkey_idkey40#1`;
- `Lotus/idkey_idkey41#1`;
- `Lotus/idkey_idkey42#1`;
- `Lotus/idkey_idkey60#1`;
- `Lotus/match_match11#1`; and
- `Microsoft/XSLTFunctions_TestOfIdFunction#1`.

## Attribution observation

Successful compilation also exposes the six principal-source DTDs that were
previously hidden behind the stylesheet pattern failure. The observed direct-
DTD attribution set grows from 120 to 126 cases: 24 bounded internal-subset
parses, 93 bounded sealed-external-subset parses, and nine explicit unsupported
declaration-semantics outcomes. This is improved attribution rather than a DTD
profile expansion; AR-0025 remains deferred.
