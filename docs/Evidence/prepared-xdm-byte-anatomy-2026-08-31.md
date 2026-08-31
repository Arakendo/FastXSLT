# Prepared-XDM Byte Anatomy

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review pressure | Adversarial review Finding 12; AR-0013 |
| Scope | One safe owned-XDM construction from an already parsed repeated-name source |
| Toolchain | `rustc 1.95.0`; `allocation-counter` 0.8.1; optimized test; x86-64 Windows 10.0.26200; AMD Family 25 Model 97 |
| Claim | Field-level retained-capacity anatomy and duplication observation; no XDM representation selected |

## Workload and method

The generated 42,039-byte source contains one `catalog` and 1,000 repeated
namespaced `item` elements. Each item has the same namespaced `code` attribute
and the same text. The resulting document contains 3,002 nodes. Parsing occurs
before the measured closure so the allocator probe attributes XDM construction,
while a private test-only anatomy walks the completed representation and sums
owned vector/string capacities by field.

The anatomy estimate includes the `Document` header, node-vector capacity,
nested relationship vectors, expanded-name strings, prefixes, values, namespace
records/strings, and per-node source-resource strings. It excludes allocator
metadata, `Arc` control-block overhead, fragmentation, and an absent whitespace
override map. It is therefore a representation capacity estimate, not process
working set and not a promise about allocator-retained bytes.

## Result

| Capacity owner | Bytes | Share of 1,223,367-byte estimate |
| --- | ---: | ---: |
| Node records | 1,015,808 | 83.0% |
| Per-node source-resource strings | 93,062 | 7.6% |
| Child and attribute relationship IDs | 72,224 | 5.9% |
| Expanded names, namespace strings, prefixes, and values | 42,017 | 3.4% |
| Namespace records | 192 | <0.1% |
| Document header | 64 | <0.1% |

Duplication observations were:

| Content | Occurrences | Unique values |
| --- | ---: | ---: |
| Local names | 2,001 | 3 |
| Namespace identities/declarations | 2,001 | 1 |
| Atomic/text values | 2,000 | 2 |
| Source-resource identity | 3,002 | 1 |

The construction requested 2,228,438 bytes across 5,029 allocations and reached
1,390,269 peak live requested bytes inside the measured closure. The probe
reported 259,033 requested bytes still live at closure exit; that allocator
counter observes only allocations made during its scope and is not expected to
equal the field-capacity estimate.

## Interpretation and disposition

The static suspicion of repeated names, namespaces, values, and resource
identity is real, but strings are not the dominant retained-capacity owner in
this deliberately repetitive shape. Node records dominate at 83%. Resource
identity interning, name interning, compact node records, and relationship
layout are now concrete AR-0013 candidates, but this single synthetic shape
does not establish their preparation cost, lookup cost, consumer break-even,
or benefit on representative documents.

No XDM representation is selected. The current safe owned tree remains the
reference. Any prototype must compare preparation latency, retained and peak
memory, warm execution, concurrency, generation ownership, node identity,
diagnostics, and host-visible behavior rather than extrapolating from duplicate
counts alone.

## Reproduction

```text
cargo test --release -p fastxslt --features allocation-observation measure_prepared_xdm_capacity_anatomy -- --ignored --nocapture
```
