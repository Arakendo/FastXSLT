# AR-0021 Versioned Incremental Protocol Admission

- Date: 2026-09-05
- Decision: ADR-0019
- Scope: private isolated-worker protocol and ASP.NET workbench adapter
- Runtime: .NET 10 preview workbench and release-mode Rust worker on Windows

## Question

Can the accepted incremental behavioral contract acquire an explicit private
protocol version without changing member semantics, weakening frame bounds, or
turning versioning into a public wire-format promise?

## Implementation

The private worker now admits a length-delimited versioned incremental command
envelope. The current implementation identifiers are operation `20`, protocol
version `1`, and acknowledgement `0x8d`; these numeric values are private test
evidence, not a public protocol registry.

The envelope contains:

1. one operation byte;
2. one little-endian `u32` protocol version;
3. one little-endian `u32` payload length; and
4. a payload bounded together with the envelope by the existing 1 MiB command
   ceiling.

The version-one payload uses one bounded controlled-member shape for both
ordinary and controlled members. Ordinary members carry explicit false control
flags rather than relying on a second versioned operation.

The worker consumes the declared bounded envelope, checks the version before
decoding any member, and returns `FXWB1005 / invalid` for an unsupported
version. Consuming the envelope before rejection preserves framing for the next
command. A version-one acknowledgement echoes the accepted version before its
member count, so the adapter does not infer compatibility merely from an
operation byte.

The existing unversioned single-request, aggregate-batch, and incremental-batch
operations remain private reference paths. No opcode, field width, frame
layout, callback, or C# type becomes a supported external contract through this
experiment.

## Results

Focused Rust tests prove:

- a version-one command decodes to the versioned incremental path;
- the acknowledgement carries version one and the admitted member count;
- an unknown version with deliberately invalid member bytes returns the typed
  unsupported-version command before member decoding; and
- the bounded rejected payload is fully consumed, allowing a following
  shutdown command to decode correctly;
- an oversized declared envelope is rejected before its payload is allocated or
  read; and
- a version-one payload with trailing bytes is rejected rather than partially
  admitted.

The ASP.NET operational gate additionally proves:

- unsupported version two returns exact `FXWB1005 / invalid`;
- the same worker completes an exact transform after that rejection;
- incremental cancellation, budget exhaustion, cumulative response exhaustion,
  backpressure, non-retaining delivery, abandonment retirement, transport-loss
  classification, and replacement recovery remain unchanged; and
- the representative four-member large-result transaction now owns exactly
  125 request wire bytes and 660,202 response wire bytes. The eight additional
  request bytes are the version and payload-length fields; the controlled
  member shape adds two explicit flag bytes per member. The four additional
  response bytes carry the acknowledged protocol version.

The prior 109-byte request and 660,198-byte response remain correct for the
unversioned experimental framing measured before ADR-0019. They are historical
evidence rather than the version-one accounting baseline.

## Validation

- `cargo test -p fastxslt-worker`: 18 passed.
- `dotnet build workbenches/FastXSLT.AspNet.Workbench/FastXSLT.AspNet.Workbench.csproj -c Release`: passed.
- `scripts/verify-aspnet-workbench.ps1 -OperationalExperiments`: passed.

The .NET SDK reports its existing preview-runtime notice. Windows may also emit
the repository's known incremental-cache hard-link fallback warning; neither
changes protocol results.

## Conclusion

ADR-0019's versioning obligation is executable. The worker rejects unknown
versions before member admission, the managed boundary verifies the echoed
version, framing remains reusable after rejection, and all established
incremental behavioral controls remain intact. Concrete wire representation
and language bindings remain private.
