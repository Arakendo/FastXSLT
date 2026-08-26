# Private Prepared-Retention Observation

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Implementation | `PreparedInputSet::observe` and `observe_totals` |
| Decision pressure | AR-0009 retained-memory classes and preparation policy |
| Claim | Private representation observation; no allocator-inclusive memory guarantee |

## Method

The prepared-input experiment now reports three separate values for each
explicitly prepared logical identity:

- retained admitted source bytes from the owning sealed snapshot;
- engine-owned XDM node count; and
- XDM-owned allocated capacity reported by the current `Document`
  representation.

Set totals sum only explicitly prepared identities. Unprepared snapshot
resources do not become parsed merely because their raw bytes are admitted.

The focused test observes the existing hello source and a generated source with
one `catalog` element, 100 `item` elements, and 100 text nodes.

## Results

| Fixture | Raw bytes | XDM nodes | XDM-owned capacity |
| --- | ---: | ---: | ---: |
| Hello | 87 | 6 | 1,932 bytes |
| Generated 100-item source | 2,109 | 202 | 63,755 bytes |

The generated node count is mechanically conserved as one document node, one
catalog element, 100 item elements, and 100 text nodes. In both cases retained
raw bytes and XDM capacity remain separately visible instead of being collapsed
into one unexplained memory number.

## Interpretation

Prepared reuse currently retains the sealed snapshot's raw bytes and a separate
owned XDM representation. The observed XDM capacity is much larger than source
bytes for these small-node-heavy inputs, so a future policy cannot budget only
input bytes or assume preparation is an in-place representation change.

The observation also supports explicit selection: preparing one identity does
not imply eager parsing of every resource admitted to the snapshot.

## Exclusions and limitations

The XDM capacity diagnostic includes the `Document` value, reserved node
storage, relationship vectors, and owned string capacity. It excludes:

- allocator metadata and fragmentation;
- `Arc` control blocks and prepared-map allocation;
- parser buffers and peak construction memory;
- compiled stylesheet storage and future derived indexes;
- invocation controls, temporary sequences, semantic results, and serialization
  buffers; and
- process, thread-stack, ASP.NET, FFI, or transport memory.

Consequently these values are implementation/build observations, not stable
formulas or host budget defaults. The workspace now has 52 tests: 50 pass and
two manual timing probes remain ignored by default.
