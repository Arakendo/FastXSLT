# OASIS XSLT 1.0 static computed-attribute namespace -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a directly constructed `xsl:attribute` retain a static namespace URI and
literal text value through the shared semantic result model, including the
prefix binding required for serialization?

## Changes

- The computed-attribute compiler accepts an optional static `namespace` URI
  for an unprefixed static NCName.
- Empty, literal-text, and the previously admitted single-`xsl:value-of`
  attribute values share the existing typed attribute representation.
- Namespace AVTs remain unsupported, and names in the reserved XMLNS namespace
  are rejected explicitly.
- A literal or statically computed owning result element retains an existing
  non-default prefix binding for the attribute namespace, or a deterministic
  collision-free binding generated at compile time.
- The binding joins the element's immutable compiled namespace slice under
  ADR-0018. The serializer does not invent mutable namespace state, and
  compiled retained-capacity accounting remains authoritative.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,525 | 1,551 | +26 |
| Executed successfully | 1,359 | 1,381 | +22 |
| Expected-result XML matches | 1,244 | 1,263 | +19 |
| XML comparison mismatches | 89 | 92 | +3 |
| Execution failures | 166 | 170 | +4 |

The three newly visible mismatches are Lotus `namespace01`, Lotus
`namespace17`, and Microsoft `78365`. The first two are annotated by the suite
for generated-prefix behavior and differ in whitespace retained between result
elements; the third uses a doubts-annotated unusual namespace URI and differs
after XML line-ending normalization. They remain uncredited. Four other cases
reach independent later runtime boundaries rather than being counted as
passes.

The strict standard-operation lower bound is now
`1,263 / 2,742 = 46.06%`; the conservative all-catalog ratio is
`1,263 / 3,173 = 39.80%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit namespace AVTs, prefixed computed-attribute names,
dynamic or nested attribute construction, namespace aliasing, or arbitrary
sequence constructors inside attributes. It does not make lexical prefixes
part of expanded-name identity. No corpus bytes or expected results changed.

## Verification

- Focused compiler tests retain the expanded name and literal value and reject
  a namespace AVT.
- A focused runtime test proves a generated binding survives result
  construction and XML serialization.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
