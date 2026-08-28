# XSLT30 Output Denominator Admission

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/output/_output-test-set.xml` |
| Discovered cases | 232 |
| Current disposition | 1 passed; 231 harness-unsupported |

## Conserved inventory

The private XSLT30 adapter now parses the complete pinned `decl/output` test
set and requires all 232 distinct native case identities. A first-party
set-level overlay applies an explicit default disposition to the immutable
complete denominator: `harness-unsupported / not-run`. One named override now
selects `output-0128` as passed. The other 231 cases remain harness-unsupported,
not engine-unsupported, because their serialization assertions or execution
adapter paths have not yet been exercised far enough to distinguish engine
behavior from harness behavior.

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

## First executable serialization case

Pinned case `output-0128` now executes through its native inline source,
stylesheet, XML-method output declaration, and file-backed
`assert-serialization` expectation. The implementation adds three engine-owned
rules exposed by that case:

- `xsl:transform` is accepted as the standard synonym for `xsl:stylesheet`;
- literal result elements retain the default namespace required by their
  expanded names, and XML serialization writes names using retained default or
  prefixed bindings; and
- `xsl:output/@include-content-type="yes"` is retained as static serialization
  metadata but does not inject an HTML `meta` element when the selected method
  is XML.

The comparator reads the upstream expected text and performs one explicit
Windows checkout correction: CRLF materialization is restored to canonical LF
before exact text comparison. It does not trim or otherwise normalize
whitespace. A focused serializer test separately proves prefixed element-name
emission and default-namespace undeclaration for an unnamespaced child.

## Claim boundary

This checkpoint proves denominator discovery, metadata classification, file
resolution by the harness, bounded memory admission, and one exact canonical
serialization comparison. It does not establish the first unsupported frontier
for the other 231 cases or claim general XML/HTML/XHTML/text serialization
conformance.

The next useful slice is widening the explicit comparator/execution adapter to
a coherent neighboring tranche. Only exercised cases may move from
`harness-unsupported` to an engine or comparison disposition.
