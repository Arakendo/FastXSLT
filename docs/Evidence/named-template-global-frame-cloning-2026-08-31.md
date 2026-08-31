# Named-Template Global-Frame Cloning

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review pressure | Adversarial review Finding 12; AR-0013 |
| Scope | Source-free warm execution with immutable atomic globals and an eight-call named-template chain |
| Toolchain | `rustc 1.95.0`; `allocation-counter` 0.8.1; optimized tests; x86-64 Windows 10.0.26200; AMD Family 25 Model 97 |
| Claim | Reference-path allocation and latency attribution; no overlay frame or alternative representation selected |

## Instrumentation and control

A test-only invocation observation records each complete global-atomic map clone
performed when a named-template call creates its frame, together with the number
of entries cloned. The hook is a no-op in ordinary builds and changes neither
variable lookup nor execution semantics.

For each global count, two compiled source-free stylesheets produce the same
single `<out/>` result:

- the depth-zero control executes initial template `t0` directly; and
- the call-chain workload executes `t0` through `t8`, adding eight named calls.

Both materialize the same globals. Their difference therefore removes the
common global-materialization and result-construction baseline. Five timed
executions produce each median. An independent `allocation-counter` observation
surrounds one complete execution on the current thread.

## Results

| Globals | Observed named-call clones | Entries cloned | Control median | Chain median | Added allocation requests | Added requested bytes | Added peak live bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 8 | 0 | 0.3 us | 1.5 us | 8 | 3,840 | 480 |
| 16 | 8 | 128 | 6.6 us | 31.2 us | 568 | 36,672 | 32,832 |
| 64 | 8 | 512 | 29.0 us | 195.5 us | 2,200 | 101,184 | 97,344 |
| 256 | 8 | 2,048 | 147.9 us | 746.2 us | 8,824 | 432,576 | 428,736 |

All measured allocations are released by closure completion. The call-chain
delta includes the named-call frame clone, the subsequent sequence-scope clone,
and fixed call execution; the explicit observation attributes eight complete
global-map clones but does not pretend every incremental byte belongs to one
line of code. The constant result and same-global control do establish that the
large deltas are invocation frame propagation rather than result growth or
global materialization.

These numbers are allocator-requested sizes, excluding allocator metadata,
rounding, fragmentation, and process memory. The timings are local mechanism
evidence, not a host-visible performance claim.

## Disposition

The global-frame half of Finding 12 advances to **confirmed; safe representation
comparison required**. A private overlay/parent-frame prototype may now be
compared against the complete-clone reference under AR-0013. It must preserve
shadowing, parameters, diagnostics, recursion limits, deterministic cleanup,
concurrency, and generation ownership, and it must include lookup cost rather
than reporting allocation reduction alone.

Prepared-XDM field duplication is a separate hypothesis and remains unmeasured.
No representation, cache, public type, or unsafe exception is admitted here.

## Reproduction

```text
cargo test -p fastxslt named_template_calls_clone_every_global_atomic_entry --all-features
cargo test --release -p fastxslt --features allocation-observation measure_named_template_global_frame_cloning -- --ignored --nocapture
```
