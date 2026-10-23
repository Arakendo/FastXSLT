# OASIS XSLT 1.0 Source Node Identity Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 607 definite unchanged XML passes; 872 initialized cases |
| Result | 608 definite unchanged XML passes; 873 initialized cases |
| Disposition | Shared source-node identity semantics; not a conformance claim |

## Change

`generate-id()` over an admitted principal-source location path now compiles to
a typed source-node identity operation. Repeated selection of the same node
returns the same non-empty identity, different nodes return different
identities, and an empty selection returns the empty string. Identity equality
uses the same operation in instruction tests and boolean-valued
`xsl:value-of` expressions.

Location-path evaluation remains work charged. The operation accepts at most
one selected node under the shared modern semantics and reports `XPTY0004` for
a larger sequence; it does not silently adopt XPath 1.0's first-node conversion
for a multi-node node-set. General legacy coercion remains behind AR-0019's
version-mode review.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 872 | 873 | +1 |
| Execution succeeded | 685 | 686 | +1 |
| XML comparison passes | 607 | 608 | +1 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **608 / 2,742 = 22.17%** of
standard-operation cases and **608 / 3,173 = 19.16%** of the complete catalog.
Unchanged OASIS `Lotus/idkey_idkey06#1` reaches a definite XML comparison
pass.

## Boundaries

This tranche covers principal-source nodes selected by the existing location
path model. Cross-document, temporary-tree, namespace-node, key-based, and
multi-node XSLT 1.0 forms remain explicit. The generated lexical form is an
engine-owned stable identity for the invocation; it does not expose resource
authority or turn a physical arena location into a public API contract.
