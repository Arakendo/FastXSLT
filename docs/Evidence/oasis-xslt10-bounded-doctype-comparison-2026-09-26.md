# OASIS XSLT 1.0 Bounded DOCTYPE Comparison

- Date: 2026-09-26
- Status: Verified corpus-harness evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The OASIS XML comparator uses the same security-hardened XML parser boundary as
the engine. That parser intentionally rejects DTD syntax. Consequently,
otherwise comparable serialized XML results containing a DOCTYPE stopped at a
parser gap even when the actual and reference declarations were identical.

This was a verification limitation. It did not justify admitting DTDs into
engine source or stylesheet parsing.

## Bounded comparison

The local-only comparator now recognizes a DOCTYPE only at the start of the
post-XML-declaration payload. A quote-aware, internal-subset-aware scanner finds
one bounded closing delimiter. The comparator then:

1. requires the actual and reference DOCTYPE declarations to be lexically
   identical;
2. removes the matching declarations only inside the comparator; and
3. parses and compares the remaining XML documents using the existing semantic
   node comparison.

An unmatched, different, or unterminated declaration cannot receive pass
credit. Production XML parsing and host DTD policy are unchanged.

## Corpus result

Eight unchanged cases now compare exactly:

- `Lotus/output_output13`;
- `Lotus/output_output15`;
- `Lotus/output_output16`;
- `Lotus/output_output18`;
- `Lotus/output_output64`;
- `Lotus/output_output65`;
- `Lotus/output_output82`; and
- `Microsoft/BVTs_bvt066`.

Fourteen additional cases leave the opaque parser-gap bucket and become visible
semantic or serialization mismatches. They are not credited and need separate
inspection. The two pre-existing doubt-annotated numeric-rendering mismatches
also remain visible.

The conserved sweep changes as follows:

- exact matches rise from 2,161 to **2,169 / 3,173 (68.36%)**;
- comparator gaps fall from 70 to **48**;
- visible mismatches rise from 2 to **16**;
- initialized cases remain **2,371**;
- successful executions remain **2,326**; and
- expected-error credit remains **423 / 431**.

The measurement emits the eight identities under the dedicated
`bounded-doctype-comparison-pass-case` disposition.

The subsequent public-only HTML DOCTYPE serializer correction adds two more
cases to that disposition. See
[HTML public-only DOCTYPE](oasis-xslt10-html-public-only-doctype-2026-09-26.md)
for the current 2,171-pass ledger.

## Boundaries

This tranche does not establish:

- production DTD parsing or entity expansion;
- semantic equivalence between different DOCTYPE declarations;
- a general HTML comparator;
- pass credit for any newly exposed mismatch; or
- a change to host XML authority or parser policy.

## Verification

```powershell
cargo test -p fastxslt oasis_xml_comparator_compares_a_bounded_doctype_before_parsing_the_document --all-features
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
