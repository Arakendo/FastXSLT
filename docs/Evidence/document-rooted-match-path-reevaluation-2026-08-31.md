# Document-Rooted Match-Path Reevaluation

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review pressure | Adversarial review Finding 11; AR-0013 |
| Scope | Private source matched-template selection for one absolute path pattern |
| Toolchain | `rustc 1.95.0`; optimized tests; x86-64 Windows 10.0.26200; AMD Family 25 Model 97 |
| Claim | Mechanism measurement only; no membership view, cache, or index selected |

## Instrumentation

A test-only invocation observation counts each document-rooted match-path
evaluation. Ordinary builds retain a no-op hook; it does not alter the path,
work budget, cancellation, template ranking, or result. Existing
`xpath-node-visit` charges supply the independently enforced work count.

The generated stylesheet broadly applies templates to sibling `item` elements
and contains one `match="/root/item"` template. Each dispatched item therefore
asks the current selector to evaluate the same absolute path from the document
node and then test membership of that item in the materialized result.

## Optimized sweep

Five complete executions were sampled per width; the table reports the median.
The budget probe then reran the same workload with exactly one fewer permitted
XPath node visit and required `FXCT0002 / limit` in the
`xpath-node-visit` domain.

| Source items | Full path evaluations | XPath node visits | One-less exhaustion limit | Median execution |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 8 | 81 | 80 | 6.1 us |
| 32 | 32 | 1,089 | 1,088 | 41.6 us |
| 128 | 128 | 16,641 | 16,640 | 379.7 us |
| 256 | 256 | 66,049 | 66,048 | 873.2 us |

For this exact source and path shape, charged visits are `(items + 1)^2`.
The counts—not the locally noisy timings—confirm repeated full-document path
evaluation and quadratic charged work. The current resource contract remains
honest because every visit is charged and the one-less probes terminate at the
advertised boundary.

## Disposition

Finding 11 advances from an open performance hypothesis to **confirmed;
representation comparison required**. A test-only safe invocation-owned
membership view may now be compared with the reference path under AR-0013. The
comparison must include construction time, peak and retained bytes, break-even
reuse, semantic and diagnostic parity, cancellation, budget accounting, and
generation ownership. This evidence does not admit caching, cross-generation
sharing, or a production index.

## Reproduction

```text
cargo test -p fastxslt document_rooted_match_path_reevaluates_for_each_dispatch_candidate --all-features
cargo test --release -p fastxslt measure_document_rooted_match_path_reevaluation --all-features -- --ignored --nocapture
```
