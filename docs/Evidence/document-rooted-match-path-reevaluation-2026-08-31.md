# Document-Rooted Match-Path Membership

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review pressure | Adversarial review Finding 11; AR-0013; ADR-0013 |
| Scope | Private source matched-template selection for one activated absolute path pattern |
| Toolchain | `rustc 1.95.0`; `allocation-counter` 0.8.1; optimized tests; x86-64 Windows 10.0.26200; AMD Family 25 Model 97 |
| Claim | Safe bounded invocation-owned membership is admitted for document-rooted match paths only |

## Reference and candidate

The reference evaluates `match="/root/item"` from the document node for every
dispatched item and linearly tests the resulting sequence. Its charged visits
are `(items + 1)^2` on this source shape.

The candidate lazily evaluates the same path once, constructs a word-backed
membership bitset keyed by compiled template index, and performs constant-time
checks for later candidates. The cache is local to `SequenceInputs`, capped at
1 MiB and 1,024 entries, and falls back to the reference without changing
semantics when either cap refuses admission. A test-only control disables the
cache while retaining the same engine path.

## Results

Five interleaved complete executions produce each median. Allocation figures
surround one independent complete execution on the current thread.

| Items | Reference evaluations / visits | Cached evaluations / visits | Builds / hits | Membership bytes | Reference / cached median | Reference / cached total requested | Reference / cached peak requested |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 8 / 81 | 1 / 18 | 1 / 7 | 8 | 6.4 / 3.8 us | 9,288 / 7,224 | 1,999 / 2,287 |
| 32 | 32 / 1,089 | 1 / 66 | 1 / 31 | 8 | 33.3 / 10.9 us | 69,264 / 29,376 | 7,776 / 8,064 |
| 128 | 128 / 16,641 | 1 / 258 | 1 / 127 | 24 | 280.3 / 38.6 us | 769,968 / 118,000 | 31,104 / 31,408 |
| 256 | 256 / 66,049 | 1 / 514 | 1 / 255 | 40 | 991.8 / 88.1 us | 2,851,120 / 236,160 | 62,208 / 62,528 |

At width 256, the safe view is about 11.3 times faster locally and requests
about 91.7% fewer total allocator bytes. Peak requested memory rises by 320
bytes because reference temporaries and the membership have overlapping
lifetime, while the membership itself is exactly 40 bytes.

The first cached construction still fails as `FXCT0002 / limit` with one fewer
permitted XPath visit. Four concurrent invocations over the same program and
prepared source each observe their own one-build/31-hit lifecycle. Direct cache
tests reject byte- and entry-ceiling overflow without mutation.

## Disposition

Finding 11 is **completed**, and ADR-0013 admits the bounded invocation-owned
membership. No prepared, global, worker, cross-snapshot, or cross-generation
cache is selected. Relative patterns and other indexes remain unchanged.

## Reproduction

```text
cargo test -p fastxslt document_rooted_match --all-features
cargo test -p fastxslt cache_refuses --all-features
cargo test --release -p fastxslt --features allocation-observation measure_document_rooted_match_path_reevaluation -- --ignored --nocapture
```
