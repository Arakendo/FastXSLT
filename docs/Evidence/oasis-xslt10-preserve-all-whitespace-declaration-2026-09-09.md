# OASIS XSLT 1.0 preserve-all whitespace declaration -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT admit the exact standalone declaration
`xsl:preserve-space elements="*"` without adding a second prepared source,
mutating prepared XDM, or implying support for selective whitespace-rule
precedence?

## Changes

- The stylesheet compiler recognizes exact `xsl:preserve-space elements="*"`
  as the existing preserve-source-whitespace policy.
- The declaration adds no runtime branch or representation: preserving all
  whitespace-only source text nodes is already the engine default.
- Selective preserve rules and any composition of preserve and strip
  declarations remain explicitly unsupported under `FXST1043`.
- The existing exact `xsl:strip-space elements="*"` visibility view and its
  complete safe reference implementation are unchanged.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,519 | 1,525 | +6 |
| Executed successfully | 1,353 | 1,359 | +6 |
| Expected-result XML matches | 1,240 | 1,244 | +4 |
| XML comparison mismatches | 87 | 89 | +2 |
| Execution failures | 166 | 166 | 0 |

Microsoft `91432`, `91433`, `91434`, and `91438` now match their unchanged
expected results. Cases `91431` and `91437` also execute with the expected node
counts, but request indented serialization and expect a newline between two
top-level result elements. FastXSLT currently emits no such indentation, so
both remain visible comparison mismatches and receive no compatibility credit.

The strict standard-operation lower bound is now
`1,244 / 2,742 = 45.37%`; the conservative all-catalog ratio is
`1,244 / 3,173 = 39.21%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit named or namespace-wildcard whitespace rules,
strip/preserve precedence, import-precedence composition, `xml:space`
interaction, or indentation behavior. It does not broaden ADR-0012's
strip-all visibility view. No corpus bytes or expected results were changed.

## Verification

- A focused compiler test proves exact preserve-all admission, the unchanged
  preserve policy, selective-rule rejection, and mixed-rule rejection.
- Six unchanged Microsoft cases reach execution; four match and two retain
  their independent indentation mismatches.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
