# Peer Review: AR-0017 Native Registry Retention

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Reviewer | Monday |
| Subject | Native handle retention, abandonment, and quota evidence |
| Outcome | Retain Incubating; distinguish legitimate live-use pressure from abandoned-handle abuse before selecting policy |

## Assessment

The review agreed that AR-0017 separates two existing ADR-0008 obligations—
bounded result/failure envelopes and atomic engine/outcome publication—from the
open aggregate retention policy. It supported repairing those contract defects
before choosing any process-wide ceiling.

The ownership constraints were retained: no silent eviction, no failure path
that requires unbounded registry capacity after exhaustion, and no registry
lock held across compilation, transformation, or foreign memory access. The
alternatives remain materially different rather than interchangeable: count
ceilings are cheap but memory-inexact; byte estimates require an accounting
model; host domains add lifecycle surface; and process isolation supplies hard
reclamation without excusing accidental trusted-host growth.

## Added measurement requirement

The review identified one calibration distinction that must remain explicit:

> Measure legitimate live-use high-water marks separately from abandonment
> pressure.

A 100,000-control abuse probe cannot establish a safe production threshold.
Representative overlapping generations, active controls, result bursts,
diagnostic bursts, and delayed-but-valid disposal need their own cardinality,
retained-byte, and whole-process-memory observations. Policy comparison must
then show whether it admits those legitimate peaks while bounding unreleased
state.

## Disposition

AR-0017 remains **Incubating**. The per-object and publication repairs proceed
under accepted ADR-0008. Aggregate count, byte, host-domain, eviction, and
isolation policy remain unselected pending separate live-use and abandonment
measurements.
