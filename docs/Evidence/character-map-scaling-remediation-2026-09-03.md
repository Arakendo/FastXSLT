# Character-Map Scaling Remediation

Date: 2026-09-03

## Question

Finding 4 of the
[second adversarial engineering review](../Reviews/adversarial-engine-review-2026-09-03.md)
identified two products of unbounded linear scans within the already bounded
stylesheet and result sizes:

- resolved-map composition searched the accumulated map for every entry; and
- serialization searched every resolved entry for every input character.

The experiment measured 100, 1,000, 5,000, and 10,000 distinct map entries.
Composition inserted each distinct entry once. Serialization wrote 10,000
unmapped ASCII characters while holding emitted byte count constant.

## Release-mode observation

The measurements are manual ignored tests and are observations on this machine,
not performance guarantees. Each value below is one release-mode observation in
microseconds; the asymptotic shape, rather than a precise ratio, decides the
remediation.

| Entries | Linear composition | Indexed composition | Linear serialization | Indexed serialization |
| ---: | ---: | ---: | ---: | ---: |
| 100 | 15 | 21 | 333 | 173 |
| 1,000 | 195 | 377 | 2,479 | 198 |
| 5,000 | 4,194 | 598 | 14,090 | 232 |
| 10,000 | 16,244 | 1,268 | 28,068 | 293 |

The prior composition curve became quadratic as distinct entries accumulated.
The prior absent-character serialization curve grew approximately in direct
proportion to map size. At 10,000 entries, indexed composition was about 12.8
times faster and indexed lookup about 95.8 times faster in this observation.
The small-map composition result also shows why this remains a focused
prepared-representation change rather than evidence for generalized indexing.

## Selected representation

Character-map resolution now uses an invocation-independent, compilation-local
`BTreeMap<char, String>` to apply inherited, local, and output-list precedence.
Compilation then retains the resolved entries as the existing compact vector,
sorted by Unicode scalar. Serialization performs binary search over that vector.

This removes the quadratic composition scan and bounds each lookup by the
logarithm of the admitted entry count without adding a hash table to every
compiled stylesheet or changing the semantic result tree. Character-map output
continues to bypass ordinary escaping exactly where the admitted serializer
already required it.

## Verification

- A focused unit test proves sorted retention and last-entry precedence.
- All 13 existing non-measurement character-map tests pass, including direct
  composition, repeated references, imported precedence, XML, XHTML, HTML, text,
  CDATA bypass, and QName identity.
- The release-mode measurement tests remain ignored during ordinary validation
  and can be rerun explicitly.
- Strict all-target/all-feature workspace Clippy passes.

No new public type, cache, global state, unsafe code, or standards claim is
introduced. The existing stylesheet resource-byte and serialized-byte ceilings
remain independently enforced.
