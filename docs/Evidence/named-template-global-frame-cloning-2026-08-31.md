# Named-Template Global-Frame Cloning

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review pressure | Adversarial review Finding 12; AR-0013 |
| Scope | Source-free warm execution with immutable atomic globals and an eight-call named-template chain |
| Toolchain | `rustc 1.95.0`; `allocation-counter` 0.8.1; optimized tests; x86-64 Windows 10.0.26200; AMD Family 25 Model 97 |
| Claim | Reference attribution plus safe shared/copy-on-write comparison admitted by ADR-0014 |

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

## Safe representation comparison

The comparison replaces the complete per-frame atomic-map clone with an
invocation-owned `Arc<BTreeMap<...>>`. Read-only frames share the map; safe
`Arc::make_mut` isolates any frame that binds or replaces a local value. The
test-only complete-clone path remains executable as the oracle. Each row below
executes the same eight-call program through both representations, compares the
complete result, and uses five interleaved timing samples plus one independent
allocation observation per path.

| Globals | Reference clone observation | Shared clone observation | Reference median | Shared median | Allocation requests saved | Requested bytes saved | Peak live bytes saved |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 8 / 0 entries | 0 / 0 entries | 1.9 us | 1.7 us | 1 | 40 | 40 |
| 16 | 8 / 128 entries | 0 / 0 entries | 17.7 us | 15.7 us | 36 | 2,092 | 2,092 |
| 64 | 8 / 512 entries | 0 / 0 entries | 84.2 us | 65.8 us | 138 | 6,124 | 6,124 |
| 256 | 8 / 2,048 entries | 0 / 0 entries | 406.8 us | 336.7 us | 552 | 26,836 | 26,836 |

The optimized path removes work monotonically as the immutable global set
grows and adds no general parent-chain lookup. Mutating frames still pay safe
copy-on-write when isolation is required. Existing parameter, shadowing,
temporary-tree, recursion, corpus, and concurrent-invocation tests cover those
semantic paths; the focused differential proves the unchanged read-only chain.

## Disposition

The global-frame half of Finding 12 is **completed**. ADR-0014 admits the private
safe shared/copy-on-write representation while retaining the complete-clone
oracle. It admits no parent-chain environment, cross-invocation sharing, public
type, or unsafe exception.

Prepared-XDM field duplication is measured separately in
[the prepared-XDM byte anatomy](prepared-xdm-byte-anatomy-2026-08-31.md); that
evidence nominates future candidates but admits no XDM representation change.

## Reproduction

```text
cargo test -p fastxslt shared_global_atomic_frames_match_the_complete_clone_reference --all-features
cargo test --release -p fastxslt --features allocation-observation measure_named_template_global_frame_cloning -- --ignored --nocapture
```
