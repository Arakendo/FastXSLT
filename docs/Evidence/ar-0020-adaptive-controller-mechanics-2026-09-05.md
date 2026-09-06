# AR-0020 Adaptive Controller Mechanics

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Scope | Test-private fixed-budget role-flexible dispatcher state machine |
| Status | Mechanics evidence only; no runtime scheduler or public contract selected |
| Implementation | `crates/fastxslt/src/runtime/preparation_pipeline_controller_tests.rs` |

## Question

Can the narrow adaptive candidate be expressed as a deterministic controller
that defaults to combined/local work, requires sustained pressure before
preparing ahead, prioritizes ready execution, and retreats under memory or
latency pressure without creating threads or exposing a preparation/execution
ratio?

## Prototype

The test-private controller accepts one fixed maximum total-worker budget and a
bounded pressure window. It returns the work class for the next available
worker:

- execute an already-ready packet;
- prepare ahead while the private mode is active;
- prepare and then execute locally in the combined baseline; or
- remain idle when no work exists.

The controller cannot create workers. Its total-worker value is retained only
as the host-supplied ceiling within which a future dispatcher would ask the
next-work question.

Preparation ahead requires three consecutive observation windows containing
raw queued work, no ready packet, and a worker waiting for execution work. A
transient observation resets the activation evidence. Once active, the
controller requires both a minimum dwell and two consecutive relief windows
before returning normally to combined/local mode.

Ready work always takes priority over preparing more work. Reaching the ready-
byte ceiling, exceeding the oldest-ready-age ceiling, or reaching the small-
request p95 ceiling causes an immediate safety retreat to combined/local mode.
Those thresholds are experiment inputs, not selected product defaults.

## Executable evidence

Five focused tests establish that:

1. transient starvation cannot activate preparation ahead;
2. sustained starvation can activate it, while a ready packet still executes
   first;
3. minimum dwell and relief hysteresis prevent a one-window mode flap;
4. byte, age, and small-tail vetoes retreat immediately; and
5. ready or empty work never asks the next worker to prepare ahead.

Focused validation:

```text
cargo test -p fastxslt preparation_pipeline_controller_tests
5 passed; 0 failed
```

The complete workspace verification also passed after this slice: formatting,
strict Clippy, 729 core tests, 18 native-boundary tests, 4 isolated-worker tests,
documentation, Markdown links, and pinned conformance-source checks.

## Interpretation

This proves only that the proposed control boundary can be made explicit and
locally auditable. It does not show that the signals correspond to a real
bottleneck, that the example window counts or veto values are appropriate, or
that adaptive staging improves throughput. The state machine is not connected
to the production dispatcher and creates no public scheduling behavior.

The prototype also exposes an activation-observability gap. A combined worker
does preparation and execution locally, so there is no distinct population of
execution workers that can report starvation. Ready-queue count, bytes, and age
likewise do not exist before staging is active. The test's
`workers_waiting_for_execution` value is therefore a synthetic controller input,
not a signal currently available from the combined reference topology.

Preparation/execution phase time can be measured in combined mode, but the
equal-thread evidence already shows it cannot select topology alone: both the
medium and large generated sources were preparation-dominant, yet their winning
topologies differed. A future connected experiment must obtain activation from
either a validated workload classifier or a bounded staged trial that includes
the cost and rollback of a losing trial. It may use ready pressure to decide
whether to remain staged, but not circularly as evidence to enter staging.

The next experiment must connect equivalent observations to the private
threaded workload and compare the controller against combined and static staged
lanes. It must include bursty phase changes and retain transition counts,
starvation, per-size latency, prepared-capacity pressure, and cancellation.
Because cache, memory-bandwidth, NUMA, allocator, and scheduler behavior can
move the break-even point, the activation rule must also be falsified on more
than one materially different machine before it can support a default.
