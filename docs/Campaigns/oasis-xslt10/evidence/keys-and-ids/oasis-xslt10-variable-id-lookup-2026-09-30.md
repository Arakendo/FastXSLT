# OASIS XSLT 1.0 variable-valued `id()` evidence

Date: 2026-09-30

## Result

The existing charged XSLT 1.0 typed-ID selector now accepts an unprefixed
variable reference as its argument. The runtime obtains the argument through
the established XSLT 1.0 scalar-conversion path, tokenizes XML whitespace,
performs typed-ID lookup, and preserves uniqueness and document order. Literal
and node-set/path arguments continue through the same selector.

The unchanged `Lotus/idkey_idkey58#1` case now initializes, executes, and
compares exactly. Its global string value `z w x` selects the three typed-ID
elements in document order, producing `WXZ`.

A focused production-path regression proves that global scalar conversion,
`for-each`, and `count(id($ids))` share the variable-valued lookup. Because
production XML parsing still denies DTDs, the regression deliberately observes
an empty typed-ID index; the unchanged OASIS case supplies the bounded DTD/XDM
integration evidence.

## Boundaries

- This adds no DTD grammar, entity, acquisition, or authority capability.
- Variable-valued match-pattern arguments remain invalid; XSLT 1.0 patterns
  still admit only literal `id()` arguments.
- The admitted compiler form is one unprefixed variable QName. Broader dynamic
  expressions remain outside this slice.
- Work charging, cancellation, uniqueness, and document-order normalization
  are unchanged.

## Measurement

The complete hash-verified local OASIS CD04 sweep reports:

- 3,173 conserved catalog cases;
- 2,465 exact XML comparisons (77.69%);
- 2,660 initialized cases;
- 2,613 successful executions;
- 510 initialization failures;
- 47 execution failures;
- 10 visible XML mismatches;
- 4 comparator gaps;
- 423 / 431 expected-error credits.

The newly executable source raises direct DTD attribution to 130 cases: 28
bounded internal parses, 93 sealed-external parses, and nine explicitly
unsupported cases. This is improved observation after AR-0025's disposition,
not a wider DTD profile.
