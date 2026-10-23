# OASIS XSLT 1.0 Malformed Path Diagnostic Classification

Date: 2026-09-28  
Status: Recorded evidence

## Question

Can FastXSLT distinguish definitely malformed XPath location-path syntax from
valid but unsupported expression families without changing the established
stylesheet-version boundary?

## Change

The shared private location-path parser now reports `XPST0003` for a bounded
set of lexically impossible forms:

- an expression beginning with an enclosed-expression delimiter;
- an unquoted semicolon or backslash; and
- adjacent numeric tokens with no operator.

Quote-aware scanning preserves those punctuation characters inside XPath
string literals. Recognized valid-but-unimplemented expression families still
remain unsupported rather than being mislabeled invalid.

The existing XSLT 1.x forward-compatible compilation boundary remains the
owner of deferral only in its selected interval above `1.0` and below `2.0`.
A declared `2.0` stylesheet remains in the modern version profile and does not
silently inherit that legacy deferral rule.

## Corpus observation

The hash-verified 3,173-case OASIS Committee Draft 04 sweep reclassifies all
nine cases formerly grouped under
`unsupported/FXXP1001/location-path-shape:other`:

- all nine malformed expressions become initialization-time `XPST0003`.

Microsoft `Errors_err046` declares version `2.0`. Its archival comment requests
runtime error timing from an XSLT 1.0 processor performing forward-compatible
processing, but FastXSLT deliberately recognizes `2.0` as a modern language
version rather than extending the private legacy interval through it. The case
still earns its catalog expected-error credit through the structured static
failure; no timing claim is inferred from the archival comment.

The conserved totals intentionally do not change: 2,228 exact matches
(70.22%), 2,439 initialized cases, 2,391 successful executions, 731
initialization failures, 48 execution failures, and 54 comparator gaps.
Expected-error credit remains 423 / 431 because the harness already credited
both unsupported and invalid structured failures. The eight uncredited catalog
entries are independently conserved as five unusable archival error
expectations and three missing archive inputs; they are not missed engine
failures.

## Verification

- Focused path-syntax classification test passes.
- The existing `1.5` forward-compatible expression-deferral regression remains
  green, proving the diagnostic change does not disturb the selected interval.
- The complete hash-verified OASIS measurement conserves all 3,173 identities
  and removes the generic `location-path-shape:other` frontier.

## Non-claims

This does not admit another XPath expression family, change pass credit, extend
legacy forward-compatible processing into declared XSLT 2.0, or establish a
general XPath lexer. It narrows diagnostic classification only where the input
is provably malformed.
