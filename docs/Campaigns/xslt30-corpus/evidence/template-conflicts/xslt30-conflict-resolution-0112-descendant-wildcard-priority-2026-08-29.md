# XSLT30 `conflict-resolution-0112` Descendant-Wildcard Priority

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0112`
- Stylesheet: `conflict-resolution-0112.xsl`
- Environment: embedded `conflict-resolution-01` principal source

## Method

The metadata-driven apply-templates helper resolves the selected case, shared
embedded source, stylesheet, and asserted XML from the pinned suite. The source
and stylesheet execute through a bounded sealed snapshot and an identified
batch of one without ambient filesystem access after admission.

The compiler recognizes only the exact `//*` lexical pattern and lowers it to a
dedicated element-applicability rule with the non-simple default priority. The
runtime selector therefore performs one node-kind test and compares the
retained priority; it does not parse the pattern or branch through general
descendant navigation during each dispatch.

A focused compiler control proves that the retained `//*` priority exceeds the
exact-name default priority. The adjacent named descendant form `//foo` remains
explicitly unsupported rather than acquiring accidental semantics.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0112` | `Match-of //* (correct)` | equal | passed |

The four compiled template rules are retained. On the principal `doc` element,
the `//*` rule beats the otherwise applicable exact `doc` and `node()` rules and
directly produces the asserted `text` result.

## Claim boundary

This evidence admits only exact `//*` as a specialized match pattern. It does
not admit `//QName`, arbitrary absolute or descendant patterns, union patterns,
general pattern grammar, ambiguity warnings, XSLT 1.0/2.0 recovery behavior, or
the complete 50-case apply-templates denominator.
