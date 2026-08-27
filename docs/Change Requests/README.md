# Change Requests

Change Requests preserve concrete needs from applications, host adapters, and
sibling projects that want to consume FastXSLT. They record the requester's
problem and required boundary without making the consumer's types,
implementation, schedule, or domain model authoritative inside the engine.

A CR may be **Proposed**, **Accepted**, **Planned**, **Implemented**,
**Deferred**, **Rejected**, or **Superseded**.

- Proposed means the request is understood well enough to review, not promised.
- Acceptance confirms that the need belongs on FastXSLT's path and links any
  architectural work it requires.
- Planning links an executable plan with acceptance evidence.
- Implementation means the accepted criteria have repeatable evidence; it does
  not imply a broader standards or compatibility guarantee.

If a request raises unresolved semantic ownership, public API, standards,
resource-authority, concurrency, ABI, or compatibility questions, open or
update an Architectural Review. If it changes an accepted boundary, record an
ADR before implementation treats that change as settled.

Use [the template](TEMPLATE.md) for new requests. Group supporting baselines,
fixtures, and plans beside the request when one consumer generates several
artifacts.

- [CR-0001: Tokimu Web3D X3D-to-VRML Transformation](CR-0001-tokimu-web3d-x3d-to-vrml-transformation.md)
  -- Deferred while Tokimu likely uses Saxon; retain as future Rust-native
  consumer pressure pending an authoritative Web3D invocation, legal fixture
  treatment, feature inventory, and AR-0012 facade evidence.
