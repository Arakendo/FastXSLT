# Prepared-engine retention estimator calibration — 2026-08-31

## Question

Can the native-lane quota experiment account for retained prepared-engine
memory more defensibly than either handle count or admitted source bytes,
without presenting an allocator observation as a stable public contract?

## Method

A feature-gated, documentation-hidden Rust workbench observation now composes
known owned capacities from one `ExperimentalEngine`:

- the engine value and retained source identity;
- prepared-map entry and identity storage;
- immutable XDM node, relationship, name, value, namespace, and provenance
  capacities; and
- recursively owned compiled stylesheet vectors, boxes, strings, names,
  locations, match patterns, instructions, temporary trees, and admitted XPath
  expression forms.

The observation does not cross the native ABI, select a quota, expose a public
representation contract, or claim allocator-exact memory. B-tree node storage,
`Arc` headers, allocator metadata, and private atomic-sequence internals remain
outside the known-capacity numerator.

An ignored release-mode test measured live requested bytes through the existing
test allocator. Unit-test builds deliberately retain the sealed input snapshot
for generation-identity tests, unlike the production workbench engine. Its
known snapshot bytes were therefore subtracted from live allocator bytes to
form a production-like comparison denominator. OS working set and private bytes
were not used as the ownership denominator.

Commands:

```text
cargo test -p fastxslt --all-features retention_estimate_is_compositional_and_scales_with_prepared_xdm
cargo test --release -p fastxslt --all-features measures_retention_estimate_against_allocator_requested_bytes -- --ignored --nocapture
```

## Results

| Shape | Known-capacity estimate | Production-like live requested bytes | Coverage |
| --- | ---: | ---: | ---: |
| `for-004`, 5 items | 12,900 | 14,185 | 90.94% |
| `for-004`, 500 items | 593,166 | 594,447 | 99.78% |
| `for-004`, 5,000 items | 4,917,937 | 4,919,219 | 99.97% |
| One 900,000-byte text node | 903,058 | 904,340 | 99.86% |
| 2,000 namespace/attribute-heavy elements | 3,077,962 | 3,079,256 | 99.96% |
| 128 matched templates | 279,455 | 280,746 | 99.54% |
| 256 global bindings | 48,533 | 49,822 | 97.41% |

The first shallow compiled-state prototype covered only 34,722 of 280,746
bytes in the template-heavy shape (12.37%). That falsification caused the
estimator to move into the compiled stylesheet owner and recursively account
for its nested private representation. The corrected model stays below the
live-allocation comparison in every measured shape while tracking source-heavy
and stylesheet-heavy growth.

The remaining gap is relatively material for tiny engines and small for large
prepared inputs. It is a known-capacity lower bound, not a safe memory ceiling:
future private representation changes can alter both its components and its
calibration.

## Conclusions

- Handle count and raw admitted bytes remain inadequate descriptions of
  prepared-engine retention.
- A compositional, representation-owned estimate can track materially different
  source and stylesheet shapes without consulting RSS.
- The estimator is suitable for replaying candidate AR-0017 admission policies
  as experimental evidence. It is not yet suitable as an enforced threshold.
- Crossing this value through the native ABI, promising its stability, or using
  it for rejection would require the quota/ABI decision AR-0017 deliberately
  leaves open.
- Consumer headroom, longer reclamation observation, and policy replay across
  sustained host-shaped workloads remain necessary before selecting a quota.
