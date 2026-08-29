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
| Selected/passed overrides | 22 |
| Default not-run dispositions | 33 |

The one secondary stylesheet belongs to `conflict-resolution-1204`. The 17
passing overrides are the previously evidenced conflict-resolution cases from
`0101` through `0901`.

This inventory also corrects the earlier provisional claim that the test set
contained 52 cases. The pinned native metadata contains exactly 50; the
executable ordered-name assertion will fail if that denominator changes.

## Claim boundary

Inventory is not conformance. The result establishes complete denominator
visibility and 22 case-specific passes only. The remaining 28 cases are not
engine failures, and no aggregate apply-templates conformance percentage is
claimed. A submodule revision change requires renewed inventory and provenance
review.
