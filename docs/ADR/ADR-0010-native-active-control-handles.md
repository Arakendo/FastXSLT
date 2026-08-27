# ADR-0010: Native Active Control Handles

- Status: Accepted
- Date: 2026-08-26
- Related reviews: AR-0002, AR-0010
- Related ADRs: ADR-0003, ADR-0008, ADR-0009
- Related evidence: `docs/Evidence/aspnet-active-cooperative-cancellation-2026-08-26.md` and `docs/Evidence/aspnet-native-active-cancellation-2026-08-26.md`
- Supersedes: None

## Context

ADR-0009 carries cancellation known before a synchronous native transform but
does not let an ASP.NET `CancellationToken` signal after Rust execution begins.
The isolated worker has executable active-cancellation evidence. AR-0002 needs
to know whether the in-process candidate can preserve the same cooperative
engine outcome without introducing callbacks, polling managed memory, or
borrowing foreign state.

A callback would require delegate rooting, reentrancy, exception containment,
thread-affinity rules, and a foreign lifetime spanning native execution. A
Rust-owned cancellation token already provides safe cloneable signalling. It
can remain behind a numeric registry handle just like engines and outcomes.

## Decision

Extend the unpublished version-zero native workbench ABI with Rust-owned
numeric control handles and these operation families:

1. create an unsignalled invocation control, optionally enabling the existing
   workbench-only first-charge barrier;
2. execute a synchronous transform with an engine handle, copied request
   identity, control handle, and scalar XSLT-instruction limit;
3. signal cancellation by control handle;
4. observe whether the experimental first-charge barrier was reached; and
5. release a control handle.

The transform clones the safe Rust cancellation state while holding the control
registry lock, releases every registry lock, and only then performs semantic
work. Cancellation, lookup, and release linearize through the same registry:

- cancellation that obtains the registry entry signals every invocation clone;
- release removes future signalling authority but does not invalidate a clone
  already owned by an executing transform;
- cancellation after release returns failure and cannot affect the invocation;
  and
- the managed adapter must retain the control handle until its transform has
  completed and any cancellation registration is disposed.

One native engine handle remains serialized by its managed wrapper. The control
operations may run concurrently with that synchronous transform and must not
acquire the managed engine gate. Independent engine handles remain the bounded
concurrency mechanism.

The first-charge barrier is test instrumentation only. It proves that a signal
can arrive after engine work begins; it is not part of latency, production
scheduling, or public cancellation semantics.

## Guarantee class

Active native cancellation is cooperative. The engine checks its Rust-owned
state at local charge points and returns `FXCT0001 / cancelled` with the logical
request identity when it observes the signal. Completion wins if the semantic
result commits before cancellation is observed.

This is not a deadline or hard-stop guarantee. A stalled charge-free native
operation, panic-abort process, foreign dependency, or OS failure cannot be
forcibly reclaimed in-process. Hard termination remains an isolated-worker
capability.

## Safety and ABI impact

Control handles are integers, never pointers. The extension adds no foreign
borrow, callback, allocator transfer, pointer arithmetic, or unsafe block. The
controlled transform reuses ADR-0008's validated immediate request-identity
copy. Safe `Arc`-backed cancellation state owns every cross-thread lifetime.

The audited first-party unsafe surface remains two unsafe blocks. Five new
exported symbols and scoped export allowances increase the exact structural
counts to fifteen export attributes and seventeen unsafe-code allowances.

Registry poisoning or handle-space exhaustion permanently quarantines the lane
under ADR-0008. Unknown/released control handles do not silently create a new
control. Repeated managed disposal is idempotent through `SafeHandle`.

## Consequences

ASP.NET can adapt a managed cancellation token without a native callback. The
synchronous P/Invoke must run away from the request thread when active managed
cancellation is required, adding task-scheduling cost that is not paid by the
ordinary synchronous hot path.

The version-zero ABI remains unpublished. This decision does not stabilize a
public cancellation type, default budget, exception mapping, async runtime, or
same-handle concurrency contract.

## Validation

- Pause a transform at its first real charge, signal it through a concurrent
  control operation, and assert the exact `FXCT0001` envelope.
- Ignore an unrelated control handle and preserve the target request.
- Execute an ordinary request on the same engine after cancellation.
- Assert release/cancel ordering and repeated managed disposal.
- Run natural unpaused cancellation races separately from the deterministic
  barrier and conserve cancellation/completion outcomes.
- Preserve direct, isolated, and native diagnostic fields while identifying
  hard termination as unavailable in-process.
- Keep the normal synchronous native throughput lane on the existing
  uncontrolled export.

Revisit this decision if cancellation checks become too sparse, a public async
ABI is proposed, one engine handle must support concurrent transforms, control
handles carry more policy, or hard containment becomes mandatory for all hosts.
