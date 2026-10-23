# Peer Review: Bounded Pre-Execution Preparation Stage

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Reviewer | Monday |
| Subject | Proposed packer thread and AR-0020 preparation-pipeline boundary |
| Outcome | Initially retained for experiment; later evidence supported rejection for the prototype path |

## Review result

The proposed packer thread is better expressed as a **preparation stage**. The
architectural question is whether pre-execution work can overlap transformation
without changing semantics, authority, or ownership; a permanent thread count
must not be selected by the terminology.

The central invariant is:

> Packing prepares immutable executable work; it does not establish new
> resource authority or new transformation semantics.

Execution packets should therefore be cheap shared ownership envelopes over
compiled and prepared state rather than copied XML/XDM or serialized private
representations. The current already-warm `for-004` benchmark is an important
negative control because another queue hop may regress its microsecond-scale
native path. A future raw-document publication workload is the relevant place
to look for parsing, XDM construction, validation, and view-preparation overlap.

The first comparison should retain three topologies: workers prepare and
execute, one preparation worker feeds ready workers, and a small bounded
preparation pool feeds them. Required observations include worker starvation,
queue wait, throughput, tail latency, retained memory, cancellation latency,
and ready-packet pressure.

Count alone is not sufficient backpressure. The experiment must record
prepared bytes outstanding in aggregate and per execution worker because packet
sizes may vary by orders of magnitude. A preparation stage that outruns
execution can otherwise become a memory amplifier even when its queue contains
only a modest number of entries.

The win condition is evidence that preparation is material, independent, and
overlap-friendly; worker utilization and end-to-end throughput improve; memory
stays inside host-owned limits; and semantics, diagnostics, authority, budgets,
cancellation, and generation ownership remain unchanged. An absent win should
remove the stage rather than preserve an extra thread whose only measured work
is moving ownership between queues.

## Scheduling-policy boundary follow-up

The equal-thread and mixed-size comparisons strengthen the case against a
public preparation/execution split. Medium inputs sometimes favored staged 7/3
or 8/2 topologies, larger inputs favored combined workers, and mixed batches
showed that staging could improve large-request tail latency while materially
worsening small-request tail latency. Submission order also changed the result.

The resulting boundary is:

> Hosts specify externally meaningful constraints. FastXSLT owns internal
> scheduling topology.

Hosts may eventually supply total concurrency, prepared-capacity limits,
cancellation/containment policy, and an evidenced service objective. They must
not be required or encouraged to select preparation workers, execution workers,
their ratio, queue structure, or worker assignment. Those are private engine
mechanics whose effects vary with workload shape and ordering.

Combined prepare-then-execute workers should remain the baseline. Staging stays
private and disabled unless FastXSLT can derive and falsify a conservative
activation rule from validated workload characteristics while staying within
the host's external limits. If no such rule survives representative testing,
the experiment should close as a useful negative result rather than shipping a
manual tuning knob.

## Dispatcher observation follow-up

The dispatcher can in principle observe preparation and execution service
times, ready count and bytes, worker busy/wait state, prepared-capacity
high-water, and oldest-ready age. Those observations might let FastXSLT detect
sustained preparation or execution pressure without exposing internal worker
roles to the host.

The first adaptive experiment should not repeatedly create and destroy threads.
It should keep one fixed host-bounded worker population whose members can work
locally in combined prepare-then-execute mode or temporarily assist preparation.
Combined/local execution is the resting state. Sustained executor starvation
may enable preparation ahead; growing ready bytes or age, saturated executors,
or deteriorating small-request latency must throttle or disable it.

Because this is a feedback controller, instantaneous queue depth is not enough.
Smoothed windows, hysteresis, minimum dwell time, transition counts, and phase-
changing workloads are required to detect oscillation. A controller that trades
memory or one latency class for an unstable throughput gain fails the experiment.
This remains a private hypothesis, not an admitted scheduler or public control.

The intended consumer-facing simplification is a single maximum total worker
budget: conceptually, "FastXSLT may use up to N workers." The dispatcher owns
whether the next available worker prepares, executes ready work, or keeps
preparation and execution local. The ceiling does not become separate per-stage
allowances and internal reassignment may never exceed it or the accompanying
prepared-memory limit.

One activation caveat remains: in the combined baseline there are no dedicated
executors to observe as starved and no ready queue whose depth, bytes, or age
can trigger staging. Those become retention/deactivation signals only after the
stage exists. Combined-mode phase timings are observable, but the measured
50/500 reversal shows their ratio alone does not predict the winning topology.
The implementation must therefore prove a pre-existing workload classifier or
use a bounded staged trial with explicit rollback; it cannot assume the
dispatcher already sees a staging bottleneck from the baseline.

The bounded replay strengthens the negative disposition. Independent
throughput, tail, and prepared-memory vetoes rejected every candidate, and the
uniform-50 reference varied enough that a short canary could make opposite
decisions for effectively the same workload. Combined/local execution remains
the prototype baseline. Adaptive staging requires a stable low-risk signal,
repeated windows, variance handling, bounded trial exposure, and a material win
on representative consumer work before it should return to the implementation
path.
