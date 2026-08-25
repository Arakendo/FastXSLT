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
artifacts. No active CR exists yet; AR-0002 records architectural questions
about a future ASP.NET consumer, but it is not a substitute for requirements
from an actual consuming application.
