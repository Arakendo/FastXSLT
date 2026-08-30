# XSLT30 Output Denominator Admission

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/output/_output-test-set.xml` |
| Discovered cases | 232 |
| Current disposition | 32 passed; 200 harness-unsupported |

## Conserved inventory

The private XSLT30 adapter now parses the complete pinned `decl/output` test
set and requires all 232 distinct native case identities. A first-party
set-level overlay applies an explicit default disposition to the immutable
complete denominator: `harness-unsupported / not-run`. Thirty-two named overrides now
select a bounded XML-compatible XHTML declaration tranche plus `output-0128`
and the XML/text cases `output-0129`, `output-0165`, `output-0166`,
`output-0171`, `output-0172`, `output-0139`, `output-0168`, and
`output-0170`, and `output-0131` as passed, together with seven standalone
lexical cases and the XHTML no-normalization control. The other 200
cases remain harness-unsupported,
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

## First executable serialization cases

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

Adjacent case `output-0129` exercises the same retained
`include-content-type="yes"` metadata under the text output method. Text
serialization walks the semantic result in document order, emits only text-node
values without XML escaping, emits no element or namespace markup, and never
injects content-type metadata. Its file-backed expectation uses the same sole
checkout-EOL correction before exact comparison. A focused nested-result test
independently proves raw descendant-text concatenation.

Two earlier cases establish a deliberately bounded XML-compatible XHTML lane.
`output-0110` verifies that `omit-xml-declaration="yes"` suppresses the
declaration, while `output-0121` verifies the XHTML default of retaining it.
Both preserve the XHTML default namespace and compare exactly with their
file-backed expectations after the same explicit checkout-EOL correction. This
lane reuses XML-compatible element/name/namespace serialization; it does not
yet claim XHTML empty-element conventions, content-type insertion, escaping,
DOCTYPE rules, or HTML-version behavior.

The declaration tranche also executes `output-0110a`, `output-0110b`,
`output-0148`, `output-0148a`, and `output-0148b`. XSLT 3.0 stylesheets accept
the whitespace-normalized boolean lexicals `true`, `false`, `1`, and `0` in
addition to `yes` and `no`. A focused negative control keeps `true` invalid for
the same property on an XSLT 2.0 stylesheet, so corpus admission does not
silently widen the older version's lexical contract.

Cases `output-0165` and `output-0166` retain `encoding="UTF-8"` with opposing
byte-order-mark settings. Case `0165` executes through a private byte-result
lane, prepends the exact three-byte UTF-8 mark `EF BB BF`, and proves the mark
is included in both the serialized-byte limit and invocation work accounting.
Its remaining bytes match the exact declared XML result required by the
upstream `all-of` assertion. Case `output-0166` retains
`byte-order-mark="no"`, executes XML serialization, proves the resulting
string has no leading BOM, and compares exactly with the upstream file. The
ordinary result boundary remains a UTF-8-compatible Rust `String` and continues
to reject `byte-order-mark="yes"`; BOM emission is evidence only on the private
bounded byte lane. Unsupported encodings remain explicit rather than mislabeled.
Focused controls cover string-lane rejection and byte-limit exhaustion.

The adjacent text-method pair `output-0171` and `output-0172` proves that BOM
handling belongs to the byte result rather than XML element serialization.
Both transform the same literal result tree into descendant text `Hello`;
`0171` produces exactly `EF BB BF 48 65 6C 6C 6F`, while `0172` produces the
five text bytes with no prefix. This executes the native cases' `all-of` and
anchored-literal intent through exact byte comparisons without admitting a
general regular-expression comparator.

The six-case XHTML family `output-0136`, `0136a`, `0136b`, `0137`, `0137a`,
and `0137b` composes the same byte-result behavior with the already admitted
XML-compatible XHTML lane. The positive cases verify the exact BOM,
declaration, XHTML namespace, and element bytes; the negative cases verify the
same bytes without the mark. XSLT 2.0 retains `yes`/`no`, while XSLT 3.0 adds
the exact `true`/`false` and whitespace-normalized `1`/`0` variants. Exact byte
comparison satisfies the native fragment assertions without widening the
harness to general regular expressions or claiming broader XHTML rules.

Case `output-0139` verifies that the UTF-8 byte lane is not merely an ASCII
metadata path. Its XML-compatible XHTML result retains `HelloÁ`, and exact byte
comparison requires the final character to appear as `C3 81` inside the full
declaration/namespace/body sequence. The case does not admit UTF-16, character
normalization, character maps, or other encoding families.

Cases `output-0168` and `output-0170` retain
`normalization-form="none"` in compiled output metadata and preserve the
literal decomposed sequence `A U+0301`. Exact byte checks require `41 CC 81`
inside XML output and as the complete text result. The paired NFC cases `0167`
and `0169` remain unsupported with `FXST1017`; FastXSLT does not substitute a
fixture-specific character mapping for a real Unicode normalization algorithm.

Case `output-0131` exercises the upstream `any-of` through its explicit
file-backed `assert-serialization` alternative. The result contains two
top-level XHTML elements separated and followed by authored text, so exact
comparison conserves result order, text, and namespace fixup without imposing a
single-document-element restriction. The harness admits only this composite
shape; it does not interpret arbitrary `any-of` trees. Adjacent `output-0173`
remains separate because it simultaneously requires output-declaration merging,
standalone metadata, and CDATA serialization.

Cases `output-0149`, `0149a`, `0149b`, `0150`, `0150a`, `0150b`, and `0152`
retain standalone metadata through compilation, bounded inspection, and XML
declaration serialization. XSLT 2.0 admits `yes`, `no`, and `omit`; XSLT 3.0
boolean and numeric lexicals canonicalize to `yes` or `no`. Exact upstream file
comparisons prove `omit` emits no standalone pseudo-attribute. This discharges
one prerequisite for `0173` without admitting declaration merging or CDATA.

Case `output-0147` composes `normalization-form="none"` with the
XML-compatible XHTML lane. Its complete byte result retains the decomposed
`41 CC 81` sequence inside the XHTML body, proving the earlier XML/text behavior
is method-independent. The NFC sibling `0146` remains unsupported pending real
Unicode normalization.

Case `output-0127` is the first passed composite serialization assertion. The
harness requires its top-level `all-of`, executes both child
`serialization-matches` assertions, and admits only a comparator subset made of
literal fragments separated by required-whitespace `\s+`. Other backslash
escapes, alternation, groups, character classes, anchors, and quantifiers remain
harness-unsupported; a focused control proves they are rejected. The case
therefore verifies XHTML `include-content-type="no"`, the XHTML namespace, and
the absence of an injected meta element without claiming a general regex
assertion engine.

## Claim boundary

This checkpoint proves denominator discovery, metadata classification, file
resolution by the harness, bounded memory admission, and thirty-two exact or
bounded-comparator upstream executions, including one byte-exact UTF-8 BOM
XML case and a paired BOM/no-BOM text control. It does not establish the first
unsupported frontier for the other 200
cases or claim general XML/HTML/XHTML/text serialization
conformance.

The next useful slice is widening the explicit comparator/execution adapter to
a coherent neighboring tranche. Only exercised cases may move from
`harness-unsupported` to an engine or comparison disposition.
