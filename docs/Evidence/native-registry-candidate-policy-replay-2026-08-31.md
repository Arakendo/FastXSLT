# Native Registry Candidate-Policy Replay

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review | AR-0017 |
| Inputs | ASP.NET generation/burst/replacement traces, 100,000-handle abandonment probe, and private prepared-engine estimator calibration |
| Claim | Arithmetic comparison of unselected policy shapes; no supported threshold or ABI behavior selected |

## Conserved observations

The component-wise legitimate high-water envelope across the current traces is:

| Dimension | Observed maximum | Source |
| --- | ---: | --- |
| Engines | 97 | Three ×32 generations plus the ordinary singleton |
| Controls | 8 | Simultaneous first-charge barrier burst |
| Outcomes | 256 | Delayed valid-result generation trace |
| Exact outcome bytes | 7,210,248 | 128 failures plus eight 900,049-byte results |
| Known prepared-engine capacity | 236,074,054 | 48 large prepared engines plus the exact ordinary singleton |

These maxima were not all simultaneous and must not be presented as a measured
single-host peak. Component-wise replay deliberately constructs a conservative
envelope that admits each observed point. The abuse probes separately retained
100,000 controls and 100,000 small outcomes.

## Candidate envelopes

Two threshold sets are replayed only to expose policy behavior:

| Candidate | Engines | Controls | Outcomes | Outcome bytes | Known engine capacity |
| --- | ---: | ---: | ---: | ---: | ---: |
| Observed-envelope count-only | 97 | 8 | 256 | none | none |
| Observed-envelope outcome hybrid | 97 | 8 | 256 | 7,210,248 | none |
| Observed-envelope full hybrid | 97 | 8 | 256 | 7,210,248 | 236,074,054 |
| Illustrative 2× full hybrid | 194 | 16 | 512 | 14,420,496 | 472,148,108 |

The observed envelope has zero operational headroom. The 2× envelope is an
arithmetic sensitivity point, not a proposed safety factor. Neither threshold
set represents consumer requirements.

## Replay

| Trace or pressure | Count-only | Outcome hybrid | Full hybrid | Illustrative 2× full hybrid |
| --- | --- | --- | --- | --- |
| Every current legitimate checkpoint | admits | admits | admits | admits |
| 100,000 controls | rejects by count | rejects by count | rejects by count | rejects by count |
| 100,000 small failure outcomes | rejects by count | rejects by count | rejects by count | rejects by count |
| Ninth 900,049-byte result added to the burst | admits | rejects by outcome bytes | rejects by outcome bytes | admits |
| 256 maximum-size outcomes | admits up to count ceiling | rejects by outcome bytes | rejects by outcome bytes | rejects by outcome bytes |
| Observed 49-engine large prepared shape | admits | admits | admits at measured envelope | admits |
| Hypothetical 97-engine large prepared shape | admits | admits | rejects by known engine capacity | admits narrowly by arithmetic |

At the observed 256-outcome count ceiling, count-only policy can admit up to
268,435,456 payload bytes because each outcome is independently bounded to one
MiB. At the illustrative 512-outcome ceiling it can admit up to 536,870,912
bytes. Therefore a count ceiling is useful abuse protection but is not a
deterministic outcome-memory bound.

The hybrid candidates distinguish the two real host shapes: 256 tiny outcomes
used only 14,080 bytes, while 136 mixed outcomes used 7,210,248 bytes. Exact
payload accounting adds useful information without pretending to account for
map allocation, allocator metadata, managed copies, or prepared engines.

The calibrated known-capacity estimator now makes the engine distinction
visible without consulting RSS. The observed 97-engine, 500-item shape accounts
for 56,957,014 known bytes, while the observed 49-engine, 5,000-item shape
accounts for 236,074,054. Replaying 97 engines of the latter shape would account
for 472,135,030 bytes. Conversely, 96 copies
of the 128-template calibration shape plus the ordinary singleton account for
26,840,758 bytes. Count therefore remains useful for abandonment but
does not predict retained engine state.

The full-hybrid replay bounds **known accounted capacity**, not allocator or
process memory. Its current lower-bound estimator cannot by itself establish a
safe total-memory ceiling. A selected policy would need explicit headroom for
unaccounted storage and representation drift, plus a decision about whether
configured engine-byte limits are versioned implementation policy or a public
host contract.

## Disposition

The replay rejects count-only as a complete memory-accounting story, but does
not reject count ceilings as one cheap dimension of a hybrid policy. Current
evidence continues to nominate:

```text
engine/control/outcome count ceilings
    + exact aggregate outcome bytes
    + private compositional known engine capacity
    + explicit host headroom for unaccounted memory
```

No threshold, margin, default, host domain, shrink rule, or exhaustion response
is selected. The mechanical evidence now differentiates all three registry
families and candidate boundary shapes. Selection is blocked on representative
consumer concurrency, overlap, memory budget, trust, and recovery requirements,
not on another invented benchmark multiplier.
