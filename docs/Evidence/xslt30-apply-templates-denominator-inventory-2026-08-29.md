# XSLT30 Apply-Templates Denominator Inventory

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- First-party overlay: `corpus/overlays/xslt30/apply-templates-denominator-v0.toml`

## Method

A dedicated inventory test parses the pinned test-set document through the
owned XML/XDM path and conserves the complete ordered case-name list. It also
counts principal and secondary stylesheets and the top-level result assertion
shapes. The overlay records a default non-execution disposition and explicit
overrides for every case currently executed through the metadata-driven helper.

The default is `harness-unsupported/not-run`, not an engine failure. An
unexecuted case may require missing engine semantics, profile handling, or
assertion support; the inventory does not guess which boundary owns it before
evidence exists.

## Result

| Observation | Count |
| --- | ---: |
| Test cases | 50 |
| Principal stylesheets | 50 |
| Secondary stylesheets | 1 |
| `assert-xml` results | 41 |
| `error` results | 8 |
| `all-of` results | 1 |
| Selected/passed overrides | 48 |
| Default not-run dispositions | 2 |

The one secondary stylesheet belongs to the passing
`conflict-resolution-1204`. The current 48 passing overrides include all eight
native error assertion shapes: two static atomic-focus failures and six
dynamic multiple-match failures. The latter require their native
`on-multiple-match=error` dependency and report concrete `XTDE0540`, satisfying
the suite's `XTRE0540` pattern. The two not-run cases are `1301` and
schema-aware `1402`; `1401` now passes through temporary-tree dispatch and
continuation.

This revision corrects stale summary values left behind as individual case
tranches advanced. The executable overlay-count assertion remained the source
of truth and now agrees with this evidence record: 48 plus 2 conserves the
50-case denominator.

This inventory also corrects the earlier provisional claim that the test set
contained 52 cases. The pinned native metadata contains exactly 50; the
executable ordered-name assertion will fail if that denominator changes.

## Claim boundary

Inventory is not conformance. The result establishes complete denominator
visibility and 48 case-specific passes only. The remaining 2 cases are not
engine failures, and no aggregate apply-templates conformance percentage is
claimed. A submodule revision change requires renewed inventory and provenance
review.
