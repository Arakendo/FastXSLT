# Native Registry Candidate-Policy Replay

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review | AR-0017 |
| Inputs | ASP.NET generation matrix, ASP.NET burst trace, and 100,000-handle abandonment probe |
| Claim | Arithmetic comparison of unselected policy shapes; no supported threshold or ABI behavior selected |

## Conserved observations

The component-wise legitimate high-water envelope across the current traces is:

| Dimension | Observed maximum | Source |
| --- | ---: | --- |
| Engines | 97 | Three ×32 generations plus the ordinary singleton |
| Controls | 8 | Simultaneous first-charge barrier burst |
| Outcomes | 256 | Delayed valid-result generation trace |
| Exact outcome bytes | 7,210,248 | 128 failures plus eight 900,049-byte results |

These maxima were not all simultaneous and must not be presented as a measured
single-host peak. Component-wise replay deliberately constructs a conservative
envelope that admits each observed point. The abuse probes separately retained
100,000 controls and 100,000 small outcomes.

## Candidate envelopes

Two threshold sets are replayed only to expose policy behavior:

| Candidate | Engines | Controls | Outcomes | Outcome bytes |
| --- | ---: | ---: | ---: | ---: |
| Observed-envelope count-only | 97 | 8 | 256 | none |
| Observed-envelope hybrid | 97 | 8 | 256 | 7,210,248 |
| Illustrative 2× count-only | 194 | 16 | 512 | none |
| Illustrative 2× hybrid | 194 | 16 | 512 | 14,420,496 |

The observed envelope has zero operational headroom. The 2× envelope is an
arithmetic sensitivity point, not a proposed safety factor. Neither threshold
set represents consumer requirements.

## Replay

| Trace or pressure | Observed count-only | Observed hybrid | 2× count-only | 2× hybrid |
| --- | --- | --- | --- | --- |
| Every current legitimate checkpoint | admits | admits | admits | admits |
| 100,000 controls | rejects by count | rejects by count | rejects by count | rejects by count |
| 100,000 small failure outcomes | rejects by count | rejects by count | rejects by count | rejects by count |
| Ninth 900,049-byte result added to the burst | admits | rejects by bytes | admits | admits |
| 256 maximum-size outcomes | admits up to count ceiling | rejects by bytes | admits | rejects by bytes |

At the observed 256-outcome count ceiling, count-only policy can admit up to
268,435,456 payload bytes because each outcome is independently bounded to one
MiB. At the illustrative 512-outcome ceiling it can admit up to 536,870,912
bytes. Therefore a count ceiling is useful abuse protection but is not a
deterministic outcome-memory bound.

The hybrid candidates distinguish the two real host shapes: 256 tiny outcomes
used only 14,080 bytes, while 136 mixed outcomes used 7,210,248 bytes. Exact
payload accounting adds useful information without pretending to account for
map allocation, allocator metadata, managed copies, or prepared engines.

Engine memory remains unresolved. A count protects cardinality but identical
engine counts can retain materially different source, XDM, and compiled state.
The current admitted-byte lower bound is not a conservative prepared-engine
estimate and cannot yet become a quota dimension.

## Disposition

The replay rejects count-only as a complete memory-accounting story, but does
not reject count ceilings as one cheap dimension of a hybrid policy. Current
evidence continues to nominate:

```text
engine/control/outcome count ceilings
    + exact aggregate outcome bytes
    + future conservative engine-retention estimate if one becomes defensible
```

No threshold, margin, default, host domain, shrink rule, or exhaustion response
is selected. Sustained consumer traces and failure-delivery experiments remain
necessary before AR-0017 can advance to a decision.
