# Private Prepared-Retention Observation

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Implementation | `ParsedDocument::owned_capacity_bytes`, `PreparedInputSet::observe`, and `observe_totals` |
| Decision pressure | AR-0009 retained-memory classes and preparation policy |
| Claim | Private representation observation; no allocator-inclusive memory guarantee |

## Method

The prepared-input experiment now reports four separate values for each
explicitly prepared logical identity:

- retained admitted source bytes from the owning sealed snapshot;
- parser-owned capacity at the completed-parse boundary, before XDM
  construction consumes the event document;
- engine-owned XDM node count; and
- XDM-owned allocated capacity reported by the current `Document`
  representation.

Set totals sum only explicitly prepared identities. Unprepared snapshot
resources do not become parsed merely because their raw bytes are admitted.

The focused test observes the existing hello source and a generated source with
one `catalog` element, 100 `item` elements, and 100 text nodes.

## Results

| Fixture | Raw bytes | Parsed-phase capacity | XDM nodes | XDM-owned capacity |
| --- | ---: | ---: | ---: | ---: |
| Hello | 87 | 938 bytes | 6 | 1,932 bytes |
| Generated 100-item source | 2,109 | 46,862 bytes | 202 | 63,755 bytes |

The generated node count is mechanically conserved as one document node, one
catalog element, 100 item elements, and 100 text nodes. In both cases retained
raw bytes, parsed-event capacity, and XDM capacity remain separately visible
instead of being collapsed into one unexplained memory number.

## Interpretation

Prepared reuse currently retains the sealed snapshot's raw bytes and a separate
owned XDM representation. The observed XDM capacity is much larger than source
bytes for these small-node-heavy inputs, so a future policy cannot budget only
input bytes or assume preparation is an in-place representation change.

The parsed-phase value is a historical phase-boundary observation. It shows
that parser-owned events are another material construction class, but it must
not be added mechanically to final XDM capacity and described as peak memory.
Ownership moves while XDM construction consumes events, and the experiment has
not observed allocator behavior or the maximum co-resident live allocation.

The observation also supports explicit selection: preparing one identity does
not imply eager parsing of every resource admitted to the snapshot.

## Exclusions and limitations

The parsed-phase diagnostic includes the `ParsedDocument`, reserved event and
root-attribute storage, and its owned name/value/resource string capacity. The
XDM diagnostic includes the `Document` value, reserved node storage,
relationship vectors, and owned string capacity. They exclude:

- allocator metadata and fragmentation;
- `Arc` control blocks and prepared-map allocation;
- `quick-xml` reader buffers and peak/co-resident construction memory;
- compiled stylesheet storage and future derived indexes;
- invocation controls, temporary sequences, semantic results, and serialization
  buffers; and
- process, thread-stack, ASP.NET, FFI, or transport memory.

Consequently these values are implementation/build observations, not stable
formulas or host budget defaults. All-feature verification now runs 57 tests:
52 pass and five manual measurement probes remain ignored by default.
