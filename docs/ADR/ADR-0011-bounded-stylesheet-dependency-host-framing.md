# ADR-0011: Bounded Stylesheet Dependency Host Framing

- Status: Accepted
- Date: 2026-08-28
- Related reviews: AR-0002, AR-0014
- Related ADRs: ADR-0002, ADR-0003, ADR-0008, ADR-0010
- Related evidence: `docs/Evidence/workbench-resource-authority-diagnostic-parity-2026-08-28.md`
- Supersedes: None

## Context

FastXSLT can compile and execute one sealed `xsl:include` dependency and can
distinguish missing resources from resources denied before membership
disclosure. The previous native and isolated failure-envelope tests constructed
that authority state inside Rust. Neither .NET host protocol could submit the
dependency bytes or denial choice, so the evidence stopped short of the real
consumer boundary.

A general resource collection ABI, callback resolver, catalog contract, or live
acquisition protocol would settle open AR-0014 questions prematurely. The
current executable language slice needs only one dependency to pressure host
framing, sealed-memory ownership, relative resolution, and diagnostic parity.

## Decision

Extend both unpublished .NET workbench initialization protocols with an
explicit one-dependency operation. The operation carries:

- the existing source and principal stylesheet identities and bytes;
- one logical dependency identity;
- bounded dependency bytes;
- a scalar admission flag; and
- an independent scalar denial flag.

Admission and denial flags accept only zero or one. An unadmitted dependency
must carry no bytes. Admission copies the dependency into the immutable sealed
snapshot before compilation. Denial is evaluated independently and before
snapshot membership disclosure, including when the same dependency was also
admitted.

Keep the original source-plus-principal initialization operations unchanged.
The new operations are explicit alternatives, not a format change silently
applied to old frames or symbols. Advance the native workbench ABI query value
to 2 so its managed adapter rejects an incompatible native artifact.

This decision does not admit a public resource API, arbitrary dependency
collections, catalogs, live callbacks, async acquisition, credentials, ambient
filesystem/network authority, cross-generation caches, or a resolver default.
It supplies evidence for AR-0014; it does not resolve that review.

## Safety and ABI impact

The native operation reuses ADR-0008's synchronous validated input-copy helper
for the additional identity and byte buffer. It retains no foreign pointer and
adds no callback, borrowed view, allocator transfer, raw handle, unsafe block,
or pointer operation. The safe engine constructor owns all copied values before
resolution and compilation begin.

One additional exported symbol and scoped export allowance increase the exact
native structural counts to sixteen export attributes and eighteen
`unsafe_code` allowances. The audited first-party unsafe operation count
remains two blocks. Invalid flags and bytes attached to an unadmitted resource
become structured boundary failures rather than engine diagnostics.

The isolated protocol adds one operation tag and uses its existing bounded
length-prefixed frames. Malformed flags or inconsistent admission framing close
the input as invalid protocol data; semantic missing and denied outcomes remain
ordinary structured initialization failures.

## Consequences

Both .NET modes can now prove the same host-visible sealed-dependency lifecycle:
admit and execute, omit and report missing, or deny and report denied. URL-shaped
logical identities remain inert identifiers and never grant network authority.

The single-dependency shape is intentionally not extensible by repetition.
Selecting a collection representation needs real multi-module cases and must
account for aggregate bytes, module occurrences, graph depth, identity aliases,
and generation ownership rather than growing this experimental call by habit.

## Validation

- Execute a principal stylesheet whose result depends on one admitted included
  module through each Rust transport implementation.
- Submit the same resolved identity as missing and denied and preserve exact
  code, category, principal include location, and detail fields.
- Reject flag values other than zero and one and reject nonempty bytes when
  admission is false.
- Build the managed ASP.NET adapter against native ABI version 2.
- Preserve the ADR-0008 through ADR-0010 pointer-copy, panic-quarantine,
  cancellation, disposal, and exact unsafe-surface gates.
- Run the normal FastXSLT verification suite.

Revisit this decision when an admitted case needs multiple modules, catalogs,
imports, runtime document access, live acquisition, credentials, or a public
host resource contract.
