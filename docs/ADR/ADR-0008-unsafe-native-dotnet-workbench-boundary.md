# ADR-0008: Unsafe Native .NET Workbench Boundary

- Status: Accepted
- Date: 2026-08-26
- Related reviews: AR-0002, AR-0004, AR-0010
- Related evidence: `docs/Evidence/aspnet-in-process-native-workbench-baseline-2026-08-26.md`
- Supersedes: None

## Context

AR-0002 requires FastXSLT to compare its proven isolated ASP.NET worker with a
leading in-process candidate before selecting a host boundary. A native library
loaded by .NET is the leading in-process candidate because it can retain the
same Rust `ExperimentalEngine`, compiled stylesheet, prepared XDM input, and
structured diagnostics without creating another semantic backend.

An actual native ABI cannot be implemented entirely in safe first-party Rust.
Rust must receive caller-owned byte pointers and copy their contents, copy
result bytes into caller-owned writable memory, and export stable symbol names.
The workspace therefore cannot build this experiment under ADR-0003's default
`unsafe_code = "forbid"` policy without a narrow exception.

The isolated worker is a safe semantic and performance reference, but it is not
an in-process mechanism. A byte-at-a-time integer ABI could avoid raw buffer
dereferences after export, but would make resource admission and result transfer
deliberately unrepresentative. Moving the unchecked code into a macro,
generated binding, C shim, or dependency would hide or enlarge the trusted
surface rather than remove the FFI invariants.

## Decision

Admit first-party unsafe code only in a new, unpublished,
workbench-only `cdylib` crate named `fastxslt-dotnet-workbench`. Keep
`unsafe_code = "forbid"` in the engine and every other existing crate. The new
crate may set `unsafe_code = "deny"`; it must not use crate-wide `allow`.
Individual reviewed functions or modules may use the minimum scoped allowance
needed for the exact surface below. `unsafe_op_in_unsafe_fn = "deny"` remains
binding.

This exception does not accept a public ABI, a production .NET package, an
execution-mode default, or unsafe code in engine semantics.

### Experimental ABI

Use C calling conventions and names prefixed `fastxslt_workbench_v0_`. Version
zero is explicitly unstable and may change or disappear between commits. Pass
only fixed-width integers, `usize` lengths, and byte pointers across the ABI;
do not expose Rust structs, enums, strings, allocators, trait objects, panics, or
raw engine addresses.

The first experiment may export only these operation families:

1. query ABI version;
2. create an engine from copied source identity/bytes and stylesheet
   identity/bytes, returning a numeric outcome handle;
3. transform using an engine handle and copied request identity, returning a
   numeric outcome handle;
4. inspect an outcome kind and byte length;
5. copy an outcome's UTF-8 result or structured diagnostic envelope into a
   caller-provided buffer;
6. take an engine handle from a successful creation outcome; and
7. release outcome and engine handles.

Numeric handles index synchronized Rust-owned registries and are never pointer
values. Every input is copied before parsing, compilation, preparation, or
execution. Every output remains Rust-owned until copied or explicitly released.
No allocator ownership crosses the boundary. No callback, resolver, borrowed
view, mapped file, ambient I/O, or managed object pointer is admitted.

Cancellation, async completion, resolver callbacks, streaming sinks, concurrent
use of one engine handle, and generation replacement are outside the first ABI.
They require further evidence and an ADR revision if they expand unsafe
lifetime or threading invariants.

### Safety contract

The managed caller establishes these invariants for every pointer call:

- a zero length permits a null pointer and performs no dereference;
- a nonzero input length accompanies a non-null pointer valid for reads of that
  many initialized bytes for the duration of the call;
- a nonzero output capacity accompanies a non-null pointer valid for writes of
  that many bytes for the duration of the call;
- the referenced allocation is not concurrently mutated, moved, freed, or
  aliased by a conflicting write while Rust copies it;
- lengths describe one allocation and do not exceed `isize::MAX` or the
  workbench's explicit resource/result bounds.

The native boundary establishes and preserves these invariants:

- validate null/length combinations, maximum lengths, handle existence, output
  capacity, and integer conversions before entering an unsafe block;
- perform no semantic work while holding a registry lock;
- clone an immutable engine reference under the lock and release the lock before
  transformation;
- never construct a slice longer than the validated length;
- never write more than the validated outcome length or caller capacity;
- never return an allocator-owned pointer or accept a caller deallocator;
- make repeated release an explicit invalid-handle outcome rather than a double
  free;
- serialize structured failures into a bounded, version-zero UTF-8 envelope;
- contain every exported body with `catch_unwind`; no panic may cross the ABI;
- after any caught panic, permanently quarantine the in-process workbench lane
  for the process. Do not reuse engine, result, or registry state and do not
  claim worker recovery.

The managed adapter pins arrays only for the synchronous duration of each copy
call and never retains native spans. Safe managed wrappers own numeric handles,
release them with `SafeHandle`, and reject use after disposal.

### Exact unsafe surface

The exception covers only:

- export attributes required to make the named C symbols visible;
- one private helper that creates a temporary input byte slice after the stated
  validation and immediately copies it into an owned `Vec<u8>`; and
- one private helper that copies a bounded Rust byte slice into a validated
  caller-owned output buffer.

Each unsafe block must have a local `SAFETY` comment tying the operation to the
contract above. Raw-pointer arithmetic, `CString::from_raw`, `Vec::from_raw_parts`,
transmute, unions, callbacks, foreign allocator adoption, raw engine handles,
and borrowed data retained after a call are not authorized. Expanding this list
requires another accepted ADR or a superseding revision.

### Panic and error behavior

Ordinary engine failures become bounded structured outcome bytes. Invalid
handles, invalid UTF-8 identities, null/length mismatches, insufficient output
capacity, and unknown outcome kinds remain machine-readable boundary failures.
The managed adapter must not parse display prose for control flow.

A caught panic is not ordinary failure containment. It atomically quarantines
the entire native lane; subsequent operations fail without touching retained
state. ASP.NET may choose another execution mode or recycle its process, but the
native layer does not silently resurrect itself.

## Consequences

The workbench can measure the same Rust semantic lifecycle in-process without
making private AST, XDM, allocator, or object layouts part of an ABI. Copying at
the boundary gives simple ownership and allows direct comparison with the
isolated worker's request/result transfer.

The design adds an auditable trusted surface, native artifact packaging, handle
registries, panic quarantine, platform testing, and managed disposal concerns.
It deliberately postpones cancellation and callbacks because those would add
cross-thread lifetime invariants before the basic candidate earns admission.

A separate crate is justified under ADR-0001 by native artifact packaging, ABI
ownership, and lint containment. It is not a new semantic engine and depends on
the same host-neutral workbench facade as the isolated worker.

## Alternatives considered

### Retain only the isolated process

This needs no unsafe first-party code and already performs well, but cannot
answer AR-0002's in-process cost, deployment, or failure-domain questions.

### Pass one byte per integer call

This can avoid raw buffer operations after symbol export, but replaces two
bounded copies with thousands of host calls. It cannot provide useful
consumer-boundary performance evidence and is rejected as a benchmark artifact.

### Hide unsafe code in generation, macros, a dependency, or a C shim

The pointer and lifetime contract still exists. Hiding it makes the exact
surface harder to audit and adds another supply-chain or language boundary.

### Expose Rust pointers as handles or transfer allocations

This reduces registry lookups but creates provenance, use-after-free,
allocator-pairing, and concurrent-release hazards with no evidence of need.

### Accept callbacks in the first ABI

Callbacks could support cancellation and resolvers, but immediately add managed
delegate rooting, thread affinity, reentrancy, panic/exception, and lifetime
contracts. The first measurement does not justify that surface.

## Validation

The implementation and every later change must satisfy all of the following
gates:

- retain the existing safe Rust facade and isolated worker as semantic
  references;
- compare byte-exact results and all structured diagnostic fields for the same
  admitted positive and negative cases;
- test empty, null/zero, null/nonzero, oversized, invalid UTF-8, unknown handle,
  released handle, repeated release, insufficient output, and concurrent
  independent-handle calls;
- run boundary tests through a separate helper process for cases that could
  terminate or corrupt the caller, and report rather than suppress a crash;
- run Miri over extractable pointer-copy/registry tests where supported and
  record the portions real FFI prevents Miri from exercising;
- run AddressSanitizer or the closest supported Windows native memory tool over
  the ABI harness; record toolchain/platform gaps explicitly;
- assert that a deliberate caught-panic probe quarantines the lane and that no
  panic unwinds into managed code;
- verify managed `SafeHandle` disposal, finalization fallback, double-dispose,
  initialization failure, and ASP.NET shutdown;
- measure cold load, creation/compile/prepare, warm transformation, result copy,
  p50/p95/p99, throughput, managed allocation, native/whole-process memory, and
  bounded concurrency against the isolated candidate;
- keep the ordinary engine crate free of unsafe code and fail CI if the allowed
  unsafe surface expands beyond the reviewed module; and
- run the normal FastXSLT verification gates plus the platform-specific ABI
  harness.

The experiment earns continued inclusion only if it supplies a materially
useful in-process capability and its measured consumer-boundary benefit or
deployment value justifies the audit surface. It does not become the default
merely by outperforming the isolated lane.

Remove the exception and crate if the in-process candidate is not selected,
fails quarantine or memory-safety verification, cannot preserve semantic and
diagnostic parity, or offers insufficient measured value. Reopen the decision
if Rust gains a safe native-export/buffer mechanism, the ABI adds callbacks or
borrowed memory, allocator ownership changes, a new platform is admitted, or
the unsafe surface must expand.
