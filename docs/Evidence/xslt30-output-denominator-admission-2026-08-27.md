# XSLT30 Output Denominator Admission

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/output/_output-test-set.xml` |
| Discovered cases | 232 |
| Current disposition | 232 harness-unsupported; 0 executed |

## Conserved inventory

The private XSLT30 adapter now parses the complete pinned `decl/output` test
set and requires all 232 distinct native case identities. A first-party
set-level overlay applies one explicit disposition to the immutable complete
denominator: `harness-unsupported / not-run`. This is intentionally not an
engine-unsupported classification because FastXSLT does not yet own the
serialization assertion comparators needed to distinguish engine behavior from
harness behavior.

The adapter conserves these top-level result assertion families:

| Assertion | Cases |
| --- | ---: |
| `all-of` | 89 |
| `any-of` | 29 |
| `assert-serialization` | 43 |
| `assert-serialization-error` | 14 |
| `error` | 6 |
| `not` | 4 |
| `serialization-matches` | 47 |
| **Total** | **232** |

It also verifies the set-wide satisfied `serialization` dependency, every
case-level specification/feature dependency, 202 referenced environments,
three inline environments, 27 source-free cases, 223 case-owned stylesheet
references, 18 resolved environment stylesheet references, seven file-backed
sources, 186 inline source instances, and 50 expected-file references.
Unknown assertion families, missing cases, duplicate identities, unresolved
environments, and missing files fail the test rather than disappearing.

## Resource boundary

For every case, the adapter maps its case-owned and resolved-environment
stylesheets and sources into a new bounded `ResourceSnapshot`. Each case is
limited to 16 resources, 64 KiB per resource, and 512 KiB total. File bytes are
read and handles closed before sealing. Inline source content is copied into
owned bytes. Logical identities include the native case identity, resource
role, ordinal, and upstream filename where present; equal bytes do not collapse
document identity.

Expected-result files remain harness-owned comparison inputs and are verified
for existence without being admitted as engine resources. The engine receives
no ambient filesystem, network, entity, or result-publication authority.

## Claim boundary

This checkpoint proves denominator discovery, metadata classification, file
resolution by the harness, and bounded memory admission. It does not execute an
XSLT30 output case, implement a serialization assertion comparator, establish
the first engine-unsupported frontier, or claim serialization conformance.

The next useful slice is comparator ownership for one coherent assertion
family and a small native case tranche. Only then can those cases move from
`harness-unsupported` to an honest execution disposition.
